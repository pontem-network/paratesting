#![feature(never_type)]
#![feature(box_patterns)]
#![feature(async_closure)]

// TODO: remove this because used in launcher mod.
extern crate subprocess;

#[macro_use]
extern crate log;

pub mod logger;
pub mod format;
pub mod error;
pub mod setup;
pub mod task;
pub mod suit;
pub mod api;
pub mod conditions;

use std::path::PathBuf;
use std::collections::BTreeMap;

use structopt::StructOpt;

use format::suit::SetupCfg;
use format::suit::PolkaLaunchCfg;
use format::suit::{ConnectCfg, NodeCfg};
use format::suit::{Test, Step, CallData};
use format::suit::Action;
use setup::{Nodes, Clients};
use client::NodeRuntimeApi;
use conditions::eval::Ctx;

pub type BoxErr = Box<dyn std::error::Error>;
pub type BoxRes<T, E = BoxErr> = color_eyre::eyre::Result<T, E>;


use client::{subxt, sp_keyring};
use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, PairSigner};
use subxt::EventsDecoder;
use subxt::ExtrinsicSuccess;
// use subxt::codec;
use subxt::ExtrinsicExtraData;

fn filter_map_for_step<'a, T>(map: &'a BTreeMap<String, T>,
                              step: &'a Step)
                              -> BoxRes<impl Iterator<Item = (&'a str, &'a T)> + 'a> {
    Ok(map.into_iter().filter_map(|(name, v)| {
                          // TODO: case-insensitive comparison
                          if step.nodes.contains(name) {
                              Some((name.as_ref(), v))
                          } else {
                              None
                          }
                      }))
}

async fn do_test_step(nodes: &Nodes, clients: &Clients, step: &Step, ctx: &mut Ctx) -> BoxRes<()> {
    let _nodes = filter_map_for_step(nodes, step)?;
    let clients = filter_map_for_step(clients, step)?;

    match &step.action {
        Action::Call { data } => {
            for (name, client) in clients {
                trace!("call for node {}", name);
                // TODO: report if error, then break the test
                let res = api::call_to_node(client, data, ctx).await?;
            }
        }

        Action::Read { .. } => {
            for (name, ..) in clients {
                trace!("read from node {}", name);
            }
            todo!()
        }

        Action::Rpc { method, .. } => {
            for (name, ..) in clients {
                trace!("send raw-rpc call `{}` to node {}", method, name);
                /*  TODO:
                    client.rpc().client
                                .request::<serde_json::Value>("rpc_methods", &[])
                */
            }
            todo!()
        }

        _ => unimplemented!(),
    }

    Ok(())
}


#[derive(StructOpt, Debug)]
struct Args {
    /// Input file or directory.
    /// For example use "examples/cases"
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}


#[async_std::main]
async fn main() -> BoxRes<()> {
    color_eyre::install()?;
    logger::init_logger();

    // TODO: use from_args_safe for github, then error!(err).
    let opt = Args::from_args();

    // TODO: get dir/file-path from args
    // let suits = suit::load_requested_suits(&PathBuf::from(TEST_SUITS_DIR))?.collect::<Vec<_>>();
    let suits = suit::load_requested_suits(&opt.input)?.collect::<Vec<_>>();
    trace!("loaded {} suits", suits.len());

    for suit in suits {
        trace!("start suit '{}'",
               suit.name.unwrap_or("unknown".to_string()));
        // let suit = TestSuite::new(suit);
        // let setup = Setup { cfg: &suit.setup };

        let mut keep_alive_proc = None::<setup::ProcState>;

        trace!("setting-up...");
        let (nodes, clients) = match suit.setup {
            SetupCfg::PolkaLaunch { cfg, conditions } => {
                let (nodes, clients, proc) = setup::run_polka_launch_proc(cfg, conditions).await?;
                // TODO: keep_alive_proc = proc;
                (nodes, clients)
            }
            SetupCfg::Process { cfg,
                                conditions,
                                connect, } => {
                let proc = setup::run_proc(cfg, conditions).await?;
                // TODO: keep_alive_proc = proc;
                //
                setup::create_clients_for_nodes(connect).await?
            }
            SetupCfg::Connect { cfg } => setup::create_clients_for_nodes(cfg).await?,
        };

        for test in suit.tests {
            info!("test {}", test.name);

            // eval context:
            let mut ctx = conditions::eval::create_test_context()?;

            for step in test.steps {
                info!("step {}", step.name);

                do_test_step(&nodes, &clients, &step, &mut ctx).await?;
            }
        }

        if let Some(proc) = keep_alive_proc {
            // send TERM
        }
    }

    Ok(())
}
