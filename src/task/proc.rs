extern crate subprocess;
use subprocess::Popen;
use subprocess::{Exec, Redirection};
use subprocess::ExitStatus;

use std::ffi::OsStr;

use async_std::future;
use futures::future::Either;
use futures::channel::oneshot;

// use cfg::ProcConditions as Conditions;
use cfg::ProcConditionsLimited as Conditions;
pub type ConditionsCfg = crate::conditions::cfg::ConditionsCfg<Conditions>;

use crate::error::{InternalError, TestError};
pub type Result<T, E = InternalError> = std::result::Result<T, E>;


pub async fn run<S>(cfg: ProcRunCfg<S>,
                    conds: ConditionsCfg)
                    -> Result<() /* TODO: <- here should be: Popen or Watcher */, TestError>
    where S: AsRef<OsStr>
{
    trace!("executing {} from {}",
           cfg.cmd.as_ref().to_str().expect("os-str repr as utf-str"),
           cfg.pwd
              .clone()
              .unwrap_or_else(|| { std::env::current_dir().unwrap_or(".".into()) })
              .display());

    let mut exec = Exec::shell(cfg.cmd.as_ref()).stdout(Redirection::Pipe)
                                                .stderr(Redirection::Pipe);
    if let Some(pwd) = cfg.pwd {
        exec = exec.cwd(pwd);
    }


    let proc = exec.popen().map_err(InternalError::from)?;
    let success = watch_proc(proc, cfg.opts, conds).await?;
    trace!("run success: {}", success);

    // TODO: do not wait if cfg.keep_alive after success
    // If failure kill it anyway

    Ok((/* Watcher or Proc */))
}

pub async fn watch_proc(mut proc: Popen,
                        opts: ProcOpts,
                        conds: ConditionsCfg)
                        -> Result</* success: */ bool, TestError> {
    // What we doing here:
    // 1. get stdout (+ try_clone fd) -> success/failure watch
    // 2. get stderr (+ try_clone fd) -> success/failure watch
    // 3. move &mut proc -> exit-watcher


    // TODO: Currently `simplify_proc_conds` makes conds for success only. Impl same for failure conditions.

    // stdout:
    let conds = simplify_proc_conds(&conds)?;
    let success_stdout = if let Some((expected, limit)) = conds.out {
        let fut = if let Some(stdout) = proc.stdout.as_mut() {
            let f = async_std::fs::File::from(stdout.try_clone().map_err(InternalError::from)?);
            let fut = watch_std(f, expected, limit);
            fut
        } else {
            return Err(InternalError::Other("unable to read stdout".to_string()).into());
        };
        Either::Left(fut)
    } else {
        Either::Right(async_std::future::pending::<Result<bool, TestError>>())
    };

    // stderr:
    let success_stderr = if let Some((expected, limit)) = conds.err {
        let fut = if let Some(stdout) = proc.stdout.as_mut() {
            let f = async_std::fs::File::from(stdout.try_clone().map_err(InternalError::from)?);
            let fut = watch_std(f, expected, limit);
            fut
        } else {
            return Err(InternalError::Other("unable to read stderr".to_string()).into());
        };
        Either::Left(fut)
    } else {
        Either::Right(async_std::future::pending::<Result<bool, TestError>>())
    };

    // TODO: don't do this, only wrapped to timeout
    let (mut exit_watcher_breaker, exit_watcher) = if opts.keep_alive.unwrap_or(false) {
        (None, Either::Left(future::ready(Ok::<_, !>((proc, None::<ExitStatus>)))))
    } else {
        let (tx, rx) = oneshot::channel::<()>();
        (Some(tx), Either::Right(async_std::task::spawn(watch_exit(proc, rx))))
    };


    use futures_lite::future::FutureExt;
    let res = success_stdout.race(success_stderr).await;
    debug!("watch proc result: {:?}", res);
    match &res {
        Ok(true) => trace!("conditions checks successfully complete"),
        Ok(false) => trace!("success conditions aren't met"),
        Err(err) => {
            trace!("setup success conditions aren't met, error: {}", err);
            if let Some(tx) = exit_watcher_breaker.take() {
                tx.send(()).ok();
                let (mut proc, exited) = exit_watcher.await.unwrap();

                if exited.is_none() {
                    proc.terminate().ok();
                }
            }
        }
    }

    // TODO: also return exit_watcher
    res
}

async fn watch_exit<T>(mut proc: Popen,
                       mut rx: oneshot::Receiver<T>)
                       -> Result<(Popen, Option<ExitStatus>), !> {
    let mut result = None;

    while let Ok(None) = rx.try_recv() {
        result = proc.poll();
        if result.is_some() {
            // exit anyway when proc closed/broken:
            break;
        }
        // TODO: exit by receive `canceled`?
        // Err(err) => return Err(InternalError::Error(Box::new(err)).into()),
    }
    Ok::<_, !>((proc, result))
}

async fn watch_std(stdout: async_std::fs::File,
                   expected: String,
                   limit: Option<WaitLimit>)
                   -> Result<bool, TestError> {
    use futures::future::FutureExt;
    use futures::TryFutureExt;
    use async_std::io::BufReader;

    let out = BufReader::new(stdout);

    let f = find_in_buf_reader(out, expected).map_err(TestError::from);
    let timed = if let Some(limit) = limit {
        let timeout = future::timeout(limit.try_into().map_err(|err: &str| TestError::from(InternalError::Other(err.to_owned())))?, f);
        Either::Left(timeout.map_err(|err| TestError::from(err)))
    } else {
        // map to looks like chained `Res` as for `timeout`
        let f = Either::Right(f.map(|ok| Ok::<_, TestError>(ok)));
        f
    };
    timed.await?
}

use io::find_in_buf_reader;
mod io {
    use super::*;
    use async_std::io::prelude::*;
    use async_std::io::BufReader;
    use async_std::stream::StreamExt;
    use async_std::fs::File;


    // TODO: split this method to two: `find_in_buf_reader` & `log_each_line`
    pub async fn find_in_buf_reader(reader: BufReader<File>,
                                    expected: String)
                                    -> Result<bool, InternalError> {
        let res = reader.lines()
                        .find(|s| {
                            if let Ok(s) = s {
                                trace!("> {}", s);
                            }

                            s.as_ref()
                             .map(|s| s.trim().contains(&expected))
                             .unwrap_or(false)
                        })
                        .await
                        .is_some();
        trace!("find in buf reader finished with found={}", res);
        Ok(res)
    }
}


struct SimplifiedProcConds {
    /// Expected and limit for __stdout__.
    out: Option<(String, Option<WaitLimit>)>,
    /// Expected and limit for __stderr__.
    err: Option<(String, Option<WaitLimit>)>,
}

fn simplify_proc_conds(conds: &ConditionsCfg) -> Result<SimplifiedProcConds, TestError> {
    let mut out = None;
    let mut err = None;
    if let Some(conditions) = conds.success.to_owned() {
        match conditions {
            Conditions::Stdout(s) => out = Some((s, None)),
            Conditions::Stderr(s) => err = Some((s, None)),
            Conditions::Wait { conditions, limit } => match conditions {
                ProcConditions::Stdout(s) => out = Some((s, Some(limit))),
                ProcConditions::Stderr(s) => err = Some((s, Some(limit))),
                _ => {
                    return Err(TestError::Feature(format!("not supported option {:?}",
                                                          conditions)))
                }
            },
            _ => return Err(TestError::Feature(format!("not supported option {:?}", conditions))),
        };
    }
    Ok(SimplifiedProcConds { out, err })
}


use super::{Task, TaskCfg, TaskConditions};
use async_trait::async_trait;

pub struct ProcTask {}

#[async_trait(?Send)]
// impl<S: AsRef<OsStr> + serde::Serialize + serde::de::DeserializeOwned> Task<ProcRunCfg<S>> for ProcTask {
impl Task<ProcRunCfg<String>> for ProcTask {
    async fn run(self,
                 cfg: ProcRunCfg<String>,
                 conds: ConditionsCfg //  conds: <ProcRunCfg<String> as TaskCfg>::Conditions
    ) -> Result<(), TestError> {
        run(cfg, conds).await
        // Ok(())
    }
}


use serde::{Serialize, de::DeserializeOwned};
impl<S: AsRef<OsStr>> TaskCfg for ProcRunCfg<S> where Self: Serialize + DeserializeOwned
{
    type Conditions = ProcConditionsLimited;
}

impl TaskConditions for ProcConditionsLimited {}


use cfg::*;
pub mod cfg {
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::collections::BTreeMap as Map;
    use serde::{Serialize, Deserialize};
    use serde_yaml::Value;
    // TODO: remove this:
    pub use crate::format::suit::ConditionsCfg;
    // pub use crate::format::suit::Conditions;
    pub use crate::format::suit::WaitLimit;


    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct ProcRunCfg<Cmd: AsRef<OsStr> = String> {
        pub pwd: Option<PathBuf>,
        pub cmd: Cmd,
        // TODO: shell
        pub opts: ProcOpts,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct ProcOpts {
        /// Keep this process alive after success/failure conditions are come.
        pub keep_alive: Option<bool>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub enum ProcConditionsLimited {
        // Result(Map<String, Value>),
        /// Check existance of all specified events
        // Events(Map<String, Value>),
        /// Check existance of event
        // Event(String),
        /// Find a line int stdout includes specified string
        Stdout(String),
        /// Find a line int stderr includes specified string
        Stderr(String),

        /// Check and compare conditions for every stream/state update
        Wait {
            #[serde(flatten)]
            conditions: ProcConditions,
            #[serde(flatten)]
            limit: WaitLimit,
        },

        /// Check and compare conditions when reqirements are met.
        /// For example, when state satisfy the reqirements.
        When {
            #[serde(flatten)]
            conditions: ProcConditions,
            #[serde(flatten)]
            reqirements: Map<String, Value>,
        },
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub enum ProcConditions {
        // Result(Map<String, Value>),
        /// Check existance of all specified events
        // Events(Map<String, Value>),
        /// Check existance of event
        // Event(String),
        /// Find a line int stdout includes specified string
        Stdout(String),
        /// Find a line int stderr includes specified string
        Stderr(String),
    }
}
