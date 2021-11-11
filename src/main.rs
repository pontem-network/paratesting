#![feature(box_patterns)]

// TODO: remove this because used in launcher mod.
extern crate subprocess;

pub mod setup;
pub mod format;
pub mod launcher;
pub mod suit;
pub mod eval;
pub mod api;

use std::path::PathBuf;
use std::collections::BTreeMap;

use format::suit::SetupCfg;
use format::suit::PolkaLaunchCfg;
use format::suit::{ConnectCfg, NodeCfg};
use format::suit::{Test, Step, CallData};
use format::suit::Action;
use launcher::{Setup, Launcher};
use setup::{Nodes, Clients};
use api::NodeRuntimeApi;
use eval::Ctx;


pub type BoxErr = Box<dyn std::error::Error>;


// TODO: XXX: remove this:
static TEST_SUITS_DIR: &str = "examples/cases";


use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, PairSigner};
use subxt::EventsDecoder;
use subxt::ExtrinsicSuccess;
// use subxt::codec;
use subxt::ExtrinsicExtraData;


fn filter_map_for_step<'a, T>(map: &'a BTreeMap<String, T>,
                              step: &'a Step)
                              -> Result<impl Iterator<Item = (&'a str, &'a T)> + 'a, BoxErr> {
	Ok(map.into_iter().filter_map(|(name, v)| {
		                  // TODO: case-insensitive comparison
		                  if step.nodes.contains(name) {
			                  Some((name.as_ref(), v))
		                  } else {
			                  None
		                  }
	                  }))
}


async fn do_test_step(nodes: &Nodes, clients: &Clients, step: &Step, ctx: &mut Ctx) -> Result<(), BoxErr> {
	let _nodes = filter_map_for_step(nodes, step)?;
	let clients = filter_map_for_step(clients, step)?;

	match &step.action {
		Action::Call { data } => {
			for (name, client) in clients {
				println!("\t call for node {}", name);
				// TODO: report if error, then break the test
				let res = api::call_to_node(client, data, ctx).await?;
			}
		},

		Action::Rpc { method, args } => {
			println!("TODO: rpc: {}({})", method, args.len());
			unimplemented!()
		},

		_ => unimplemented!(),
	}


	Ok(())
}


#[async_std::main]
async fn main() -> Result<(), BoxErr> {
	// env_logger::init();

	// TODO: get dir/file-path from args
	let suits = suit::load_requested_suits(&PathBuf::from(TEST_SUITS_DIR))?.collect::<Vec<_>>();
	println!("loaded {} suits", suits.len());


	for suit in suits {
		// let suit = TestSuite::new(suit);
		// let setup = Setup { cfg: &suit.setup };


		let (nodes, clients) = match suit.setup {
			SetupCfg::PolkaLaunch { cfg, conditions } => {
				let mut launcher = Setup::<PolkaLaunchCfg>::new(cfg, conditions);
				launcher.run();
				// TODO: if success => continue to SetupCfg::Connect(build ConnectCfg)
				todo!()
			},
			SetupCfg::Process { .. } => {
				todo!()
				// TODO:
				// let mut launcher = Setup::<ProcessRunCfg>::new(cfg, conditions);
				// launcher.run();
			},
			SetupCfg::Connect { cfg } => setup::create_clients_for_nodes(cfg).await?,
		};

		for test in suit.tests {
			println!("test {}", test.name);

			// eval context:
			let mut ctx = eval::create_test_context()?;

			for step in test.steps {
				println!("step {}", step.name);

				do_test_step(&nodes, &clients, &step, &mut ctx).await?;
			}
		}
	}

	return Ok(());

	/*
		client.rpc()
					.client
					.request::<serde_json::Value>("rpc_methods", &[])


		// event:
		if let Some(event) = result.find_event::<polkadot::balances::events::Transfer>()? {
			println!("Balance transfer success: value: {:?}", event.2);
		} else {
			println!("Failed to find Balances::Transfer Event");
		}
	*/


	// let ws = "ws://127.0.0.1";
	// let url_alice = format!("{}:{}", ws, 9944);
	// let url_bob = format!("{}:{}", ws, 9945);
	// let url_pontem = format!("{}:{}", ws, 9946);

	// // let signer = PairSigner::new(AccountKeyring::Alice.pair());
	// // let dest = AccountKeyring::Bob.to_account_id().into();

	// {
	// 	// Pontem
	// 	{
	// 		let mut client = ClientBuilder::new().set_url(&url_pontem)
	// 		                                     .build::<pontem::DefaultConfig>()
	// 		                                     .await?;

	// 		// let meta = client.rpc().metadata().await?;
	// 		// println!("pontem.meta: {:#?}", meta);


	// 		let api = client.to_runtime_api::<pontem::RuntimeApi<_>>();
	// 		let mut iter = api.storage().system().account_iter(None).await?;
	// 		while let Some((key, account)) = iter.next().await? {
	// 			println!("{}: {}", hex::encode(key), account.data.free);
	// 		}

	// 		println!("pontem.move STORAGE:");
	// 		let mut iter = api.storage().mvm().vm_storage_iter(None).await?;
	// 		while let Some((key, value)) = iter.next().await? {
	// 			println!("{}: {}", hex::encode(key), hex::encode(value));
	// 		}
	// 	}
	// }

	Ok(())
}
