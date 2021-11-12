use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, PairSigner};
use subxt::EventsDecoder;
use subxt::ExtrinsicSuccess;
// use subxt::codec;
use subxt::ExtrinsicExtraData;
use subxt::sp_core::crypto::Ss58Codec;
use subxt::sp_core;

use crate::BoxErr;
use crate::format;
use format::suit::SetupCfg;
use format::suit::PolkaLaunchCfg;
use format::suit::{ConnectCfg, NodeCfg};
use format::suit::{Test, Step, CallData};
use format::suit::Action;
use crate::eval::Ctx;


#[subxt::subxt(runtime_metadata_path = "nodes/polkadot_metadata.scale",
               generated_type_derives = "Clone, Debug")]
pub mod polkadot {}

// #[subxt::subxt(runtime_metadata_path = "nodes/pontem-xcmp.scale")]
// #[subxt::subxt(runtime_metadata_path = "nodes/pontem.scale")]
pub mod pontem;

#[subxt::subxt(runtime_metadata_path = "nodes/rococo-local.scale",
               generated_type_derives = "Clone, Debug")]
pub mod rococo {}


pub enum NodeRuntimeApi {
	Pontem(pontem::RuntimeApi<pontem::DefaultConfig>),
	Polkadot(polkadot::RuntimeApi<polkadot::DefaultConfig>),
	Rococo(rococo::RuntimeApi<rococo::DefaultConfig>),
}

impl NodeRuntimeApi {
	pub async fn new(cfg: &NodeCfg) -> Result<Self, BoxErr> {
		use format::suit::SupportedRuntime;

		// TODO: support custom ip
		let ws = "ws://127.0.0.1";
		let url = format!("{}:{}", ws, cfg.port);

		let client = match cfg.runtime {
			SupportedRuntime::Pontem => {
				NodeRuntimeApi::Pontem(ClientBuilder::new().set_url(&url)
				                                           .build::<pontem::DefaultConfig>()
				                                           .await?
				                                           .to_runtime_api::<pontem::RuntimeApi<_>>())
			},
			SupportedRuntime::Rococo => {
				NodeRuntimeApi::Rococo(ClientBuilder::new().set_url(&url)
				                                           .build::<rococo::DefaultConfig>()
				                                           .await?
				                                           .to_runtime_api::<rococo::RuntimeApi<_>>())
			},
			SupportedRuntime::Polkadot => {
				NodeRuntimeApi::Polkadot(ClientBuilder::new().set_url(&url)
				                                             .build::<polkadot::DefaultConfig>()
				                                             .await?
				                                             .to_runtime_api::<polkadot::RuntimeApi<_>>())
			},
		};

		Ok(client)
	}
}


pub async fn call_to_node(client: &NodeRuntimeApi, call: &CallData, ctx: &mut Ctx) -> Result<(), BoxErr> {
	let (a, b, c) = {
		let ref abc = call.method.split('.').collect::<Vec<&str>>()[..3];
		(abc[0], abc[1], abc[2])
	};

	// TODO: // eval_with_context(method, ctx)?;


	macro_rules! same_for_all {
		// `()` indicates that the macro takes no argument.
		($client:ident, | | $block:expr) => { same_for_all!($client, || $block) };
		($client:ident, || $block:expr) => {
			match $client {
				NodeRuntimeApi::Pontem(..) => $block,
				NodeRuntimeApi::Rococo(..) => $block,
				NodeRuntimeApi::Polkadot(..) => $block,
			}
		};
		($client:ident, |$($api:ident),+| $block:expr) => {
			match $client {
				NodeRuntimeApi::Pontem($($api)*, ..) => $block,
				NodeRuntimeApi::Rococo($($api)*, ..) => $block,
				NodeRuntimeApi::Polkadot($($api)*, ..) => $block,
			}
		};
	}


	match a {
		"tx" => {
			match b {
				"balances" => {
					match c {
						"transfer" => {
							println!("TRANSFER: {}({:?})", call.method, call.args);

							// TODO: ensure that call.args.len() == 2;

							// TODO: ensure that call.signer.is_some
							// let signer = call.signer.as_ref().expect("signer should be");

							// TODO: eval dest, amount & signer

							// :subxt::sp_runtime::MultiAddress<subxt::sp_runtime::AccountId32, ()>

							// let dest: subxt::sp_runtime::MultiAddress<subxt::sp_runtime::AccountId32, ()> =
							// 	AccountKeyring::Bob.to_account_id().into();


							same_for_all!(client, |api| {
								// let dest = AccountKeyring::Bob.to_account_id().into();
								// println!("dest: {:?}", dest);

								// let signer = PairSigner::new(AccountKeyring::Alice.pair());
								// // let signer: subxt::PairSigner<
								// //                               crate::api::pontem::api::DefaultConfig,
								// //                               sp_keyring::sr25519::sr25519::Pair,
								// // > = crate::eval::signer_from_str(&signer)?;
								// let result = api.tx()
								//                 .balances()
								//                 .transfer(dest, 10_000)
								//                 .sign_and_submit_then_watch(&signer)
								//                 .await?;
								// println!("RESULT: {} :: {}", result.block, result.extrinsic);

								// // check result
								// // event:
								// let e = result.find_event::<pontem::balances::events::Transfer>();
								// println!("EVENT: {:#?}", e);

								// const ID: &str = "Balances::Transfer";

								// if let Some(event) = e? {
								// 	println!("Balance transfer success: value: {:?}", event.2);

								// 	// crate::eval::add_events_to_context(ctx, &result.events)?;

								// 	let pontem::balances::events::Transfer(from, to, amount) = event;
								// // let from = format!("{}", from.to_ss58check());
								// // let to = format!("{}", to.to_ss58check());
								// // use evalexpr::*;
								// // ctx.set_value(format!("{}.from", ID), Value::from(from))?;
								// // ctx.set_value(format!("{}.to", ID), Value::from(to))?;
								// // ctx.set_value(format!("{}.amount", ID), Value::from(amount as i64))?;
								// } else {
								// 	println!("Failed to find Balances::Transfer Event");
								// 	// TODO: should fail the step
								// }
							});
						},
						_ => unimplemented!(),
					}
				},
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	}

	Ok(())
}
