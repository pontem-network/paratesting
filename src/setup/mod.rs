use std::ffi::OsStr;
use std::collections::BTreeMap;
use std::path::{PathBuf, Path};
use std::io::BufRead;
use std::io::BufReader;

use client::NodeRuntimeApi;

use crate::BoxErr;
use crate::BoxRes;
use crate::format;
use format::suit::{ConditionsCfg, Conditions};
use format::suit::{ConnectCfg, NodeCfg, PolkaLaunchCfg, ProcessRunCfg};
use client::SupportedRuntime;

use subprocess::Popen;
use subprocess::{Exec, Redirection};

pub type Nodes = BTreeMap<String, NodeCfg>;
pub type Clients = BTreeMap<String, NodeRuntimeApi>;
pub type ProcState = ();

pub async fn create_clients_for_nodes(cfg: ConnectCfg) -> BoxRes<(Nodes, Clients)> {
    let mut nodes = BTreeMap::new();

    for node in cfg.nodes.into_iter() {
        nodes.insert(node.name.to_owned(), node.clone());
    }

    let clients = make_clients(&nodes).await?;

    Ok((nodes, clients))
}

pub async fn make_clients(nodes: &Nodes) -> BoxRes<Clients> {
    let mut clients = BTreeMap::new();
    for (name, node) in nodes.iter() {
        // TODO: support custom ip
        let ws = "ws://127.0.0.1";
        let url = format!("{}:{}", ws, node.port);
        clients.insert(name.to_owned(),
                       NodeRuntimeApi::new(url, node.runtime).await?);
    }
    Ok(clients)
}

pub async fn run_polka_launch_proc(cfg: PolkaLaunchCfg,
                                   conditions: ConditionsCfg)
                                   -> BoxRes<(Nodes, Clients, ProcState)> {
    let cmd = format!("polkadot-launch {}", &cfg.cfg.display());

    let nodes = {
        let path = cfg.inner
                      .pwd
                      .as_ref()
                      .map(|pwd| pwd.join(&cfg.cfg))
                      .unwrap_or(cfg.cfg);
        load_polka_launch_cfg(&path).await?
    };

    let proc_cfg = ProcessRunCfg { cmd,
                                   pwd: cfg.inner.pwd.to_owned(),
                                   keep_alive: cfg.inner.keep_alive.unwrap_or(true) };

    let proc = run_proc(proc_cfg, conditions).await?;
    let clients = make_clients(&nodes).await?;

    Ok((nodes, clients, (/* ProcState */)))
}

pub async fn run_proc<S>(cfg: ProcessRunCfg<S>, conditions: ConditionsCfg) -> BoxRes<ProcState>
    where S: AsRef<OsStr> {
    let mut exec = Exec::shell(cfg.cmd.as_ref()).stdout(Redirection::Pipe)
                                                .stderr(Redirection::Pipe);
    if let Some(pwd) = cfg.pwd {
        exec = exec.cwd(pwd);
    }

    let res = watch_proc(exec.popen()?, conditions, cfg.keep_alive).await?;

    // TODO: do not wait if cfg.keep_alive after success
    // If failure kill it anyway

    Ok((/* ProcState */))
}

/* WATCHER */
pub async fn watch_proc(mut p: Popen, conditions: ConditionsCfg, keep_alive: bool) -> BoxRes<()> {
    let mut out_conds = None;
    let mut err_conds = None;
    if let Some(conditions) = conditions.success {
        match conditions {
            Conditions::Stdout(s) => out_conds = Some((s, None)),
            Conditions::Stderr(s) => err_conds = Some((s, None)),
            Conditions::Wait { box conditions,
                               limit, } => match conditions {
                Conditions::Stdout(s) => out_conds = Some((s, Some(limit))),
                Conditions::Stderr(s) => err_conds = Some((s, Some(limit))),
                _ => unimplemented!("not supported option {:?}", conditions),
            },
            _ => unimplemented!("not supported option {:?}", conditions),
        };
    }

    // use async_std::future::*;
    use async_std::future::Future;
    use async_std::future;
    use futures::future::FutureExt;
    use futures::future::Either;
    use futures::TryFutureExt;

    unsafe impl Send for ProcWatchError {}
    custom_error::custom_error! {pub ProcWatchError
         StdoutUnavailable = "unable to read stdout",
         StderrUnavailable = "unable to read stderr",
         Fail { reason: String } = "fail: {reason}"
    }

    let success = if let Some((expected, limit)) = out_conds {
        let stdout = p.stdout.as_mut();
        let mut out = stdout.map(|s| BufReader::new(s));

        let f = if let Some(out) = out.take() {
            let f = find_in_buf_reader(out, expected);
            let timed = if let Some(limit) = limit {
                let timeout = future::timeout(limit.try_into()?, f);
                Either::Left(timeout.map_err(|err| BoxErr::from(err)))
            } else {
                // map to looks like chained `Res` as for `timeout`
                Either::Right(f.map(|ok| Ok::<_, BoxErr>(ok)))
            };
            Either::Left(timed)
        } else {
            Either::Right(futures::future::err::<BoxRes<bool>, BoxErr>(BoxErr::from(ProcWatchError::StdoutUnavailable)))
        };
        Either::Left(f)
    } else {
        Either::Right(future::pending::<BoxRes<BoxRes<bool>>>())
    };

    let failure = if let Some((expected, limit)) = err_conds {
        let stderr = p.stderr.as_mut();
        let mut err = stderr.map(|s| BufReader::new(s));

        let f = if let Some(out) = err.take() {
            let f = find_in_buf_reader(out, expected);
            let timed = if let Some(limit) = limit {
                let timeout = future::timeout(limit.try_into()?, f);
                Either::Left(timeout.map_err(|err| BoxErr::from(err)))
            } else {
                // map to looks like chained `Res` as for `timeout`
                Either::Right(f.map(|ok| Ok::<_, BoxErr>(ok)))
            };
            Either::Left(timed)
        } else {
            Either::Right(futures::future::err::<BoxRes<bool>, BoxErr>(BoxErr::from(ProcWatchError::StdoutUnavailable)))
        };
        Either::Left(f)
    } else {
        Either::Right(future::pending::<BoxRes<BoxRes<bool>>>())
    };

    // use futures::future::FutureExt;
    let success = success.into_future();
    {
        use futures_lite::future::FutureExt;
        let res = success.race(failure).await;
        println!("RACE RESULT: {:?}", res);

        match res {
            Err(err) | Ok(Err(err)) => {
                p.terminate().ok();
                println!("setup success conditions aren't met, error: {}", err);
                return Err(err);
            }
            Ok(Ok(false)) => {
                p.terminate().ok();
                let reason = "setup success conditions aren't met".to_owned();
                println!("{}", reason);
                return Err(ProcWatchError::Fail { reason }.into());
            }
            Ok(Ok(true)) => {
                if !keep_alive {
                    p.terminate().ok();
                }
                println!("setup successfully complete");
            }
        }
    }

    // let (res, i, remaining) = futures::future::select_all([Either::Left(success), Either::Right(failure)]).await;
    // let (res, i, remaining) = futures::future::select_all([success.into_future(), failure.into_future()]).await;
    // println!("SELECT RESULT: {:?} ({}/{})", res, i, remaining.len());

    Ok(())
}

/* WATCHER methods */

/// FIXME: There's potential bug and block because of blocking io.
/// We should use async_std::BufReader -> Lines -> Stream -> Find.
pub async fn find_in_buf_reader(reader: BufReader<&mut std::fs::File>,
                                expected: String)
                                -> Result<bool, BoxErr> {
    let res = reader.lines()
                    .find(|s| {
                        println!("{:?}", s);
                        s.as_ref()
                         .map(|s| s.trim().contains(&expected))
                         .unwrap_or(false)
                    })
                    .is_some();
    println!("END! success: {}", res);
    Ok(res)
}
/* WATCHER END */

pub async fn load_polka_launch_cfg(path: &Path) -> Result<Nodes, BoxErr> {
    let f = std::fs::File::open(path)?;
    let cfg: format::launch::Config = format::launch::deserialize(f)?;

    let mut nodes = BTreeMap::new();

    for node in cfg.relaychain.nodes.into_iter() {
        let name = node.name
                       .as_ref()
                       .cloned()
                       .unwrap_or_else(|| format!("{}", node.ws_port));

        let runtime = if cfg.relaychain.chain.to_lowercase().contains("pontem")
                         || cfg.relaychain.bin.to_lowercase().contains("pontem")
        {
            SupportedRuntime::Pontem
        } else if cfg.relaychain.chain.to_lowercase().contains("rococo")
                  || cfg.relaychain.bin.to_lowercase().contains("rococo")
        {
            SupportedRuntime::Rococo
        } else {
            SupportedRuntime::Polkadot
        };
        // TODO: check log exists
        let log_file = node.name
                           .map(|s| PathBuf::from(format!("{}.log", s.to_lowercase())))
                           .unwrap_or_else(|| PathBuf::from(format!("{}.log", node.ws_port)));

        nodes.insert(name.clone(),
                     NodeCfg { name,
                               runtime,
                               port: node.ws_port,
                               log_file: Some(log_file) });
    }

    for chain in cfg.parachains.into_iter() {
        for node in chain.nodes.into_iter() {
            let name = node.base
                           .name
                           .as_ref()
                           .cloned()
                           .unwrap_or_else(|| format!("{}", node.base.ws_port));

            let runtime = if chain.bin.to_lowercase().contains("pontem") {
                SupportedRuntime::Pontem
            } else if chain.bin.to_lowercase().contains("rococo") {
                SupportedRuntime::Rococo
            } else {
                SupportedRuntime::Polkadot
            };

            // TODO: check log exists
            let log_file =
                node.base
                    .name
                    .map(|s| PathBuf::from(format!("{}.log", s.to_lowercase())))
                    .unwrap_or_else(|| PathBuf::from(format!("{}.log", node.base.ws_port)));

            nodes.insert(name.clone(),
                         NodeCfg { name,
                                   runtime,
                                   port: node.base.ws_port,
                                   log_file: Some(log_file) });
        }
    }

    for node in cfg.simple_parachains.into_iter() {
        let runtime = if node.bin.to_lowercase().contains("pontem") {
            SupportedRuntime::Pontem
        } else if node.bin.to_lowercase().contains("rococo") {
            SupportedRuntime::Rococo
        } else {
            SupportedRuntime::Polkadot
        };

        // TODO: check log exists
        let log_file = PathBuf::from(format!("{}.log", node.port));

        nodes.insert(node.bin.clone(),
                     NodeCfg { runtime,
                               name: node.bin,
                               port: node.port,
                               log_file: Some(log_file) });
    }

    Ok(nodes)
}
