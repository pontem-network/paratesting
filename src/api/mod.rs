// TODO: move all of this out from the crate - to the client/api crate.

use client::subxt;
use subxt::{ClientBuilder, PairSigner};
use subxt::EventsDecoder;
use subxt::ExtrinsicSuccess;
// use subxt::codec;
use subxt::ExtrinsicExtraData;
use subxt::sp_core::crypto::Ss58Codec;
use subxt::sp_core;

use futures::FutureExt;
use futures::TryFutureExt;

use crate::BoxErr;
use crate::format;
use format::suit::SetupCfg;
use format::suit::PolkaLaunchCfg;
use format::suit::{ConnectCfg, NodeCfg};
use format::suit::{Test, Step, CallData};
use format::suit::Action;
use crate::conditions::eval::Ctx;
use crate::error::TestError;


use client::{pontem, polkadot, rococo};
use client::{SupportedRuntime, NodeRuntimeApi};

pub async fn call_to_node(client: &NodeRuntimeApi,
                          call: &CallData,
                          ctx: &mut Ctx)
                          -> Result<(), TestError> {
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
				NodeRuntimeApi::Pontem(..) => {#[allow(unused_imports)] use pontem as api_current; $block},
				NodeRuntimeApi::Rococo(..) => {#[allow(unused_imports)] use rococo as api_current; $block},
				NodeRuntimeApi::Polkadot(..) => {#[allow(unused_imports)] use polkadot as api_current; $block},
			}
		};
		($client:ident, |$($api:ident),+| $block:expr) => {
			match $client {
				NodeRuntimeApi::Pontem($($api)*, ..) => {#[allow(unused_imports)] use pontem as api_current; $block},
				NodeRuntimeApi::Rococo($($api)*, ..) => {#[allow(unused_imports)] use rococo as api_current; $block},
				NodeRuntimeApi::Polkadot($($api)*, ..) => {#[allow(unused_imports)] use polkadot as api_current; $block},
			}
		};
	}

    match a {
        "storage" => match b {
            "system" => match c {
                "account" => {
                    same_for_all!(client, |api| {
                        let mut iter = api.storage().system().account_iter(None).await?;
                        while let Some((key, account)) = iter.next().await? {
                            // println!("{}: {}, nonce: {}, ", hex::encode(&key), account.data.free, account.nonce);
                            println!("{:?}", account);
                            // AccountInfo { nonce: 0, consumers: 1, providers: 1, sufficients: 0, data: AccountData { free: 899990000000000, reserved: 100010000000000, misc_frozen: 100000000000000, fee_frozen: 0 } }
                        }

                        let dest = keys::id_from_str("Alice").unwrap();
                        let info = api.storage().system().account(dest, None).await?;
                        println!("Alice: {:?}", info);
                    });
                }
                _ => unimplemented!("not supported option {}", c),
            },
            "mvm" => match c {
                "vm_storage" | "vm-storage" => {
                    // match client {
                    //     NodeRuntimeApi::Pontem(api, ..) => {
                    //         println!("pontem.move STORAGE:");
                    //         let mut iter =
                    //             api.storage().mvm().vm_storage_iter(None).await?;
                    //         while let Some((key, value)) = iter.next().await? {
                    //             println!("{}: {}", hex::encode(key), hex::encode(value));
                    //         }

                    //         //
                    //         println!("pontem.move STORAGE::28e050611b6cb9358721c8...");
                    //         const KEY:&str = "28e050611b6cb9358721c8a9dc75e9420732facf0c850d01d69e1e24eecdc59d173e987efeeb8fb5ef42b68733193845a0000000000000000000000000000000000000000000000000000000000000000001064469656d4964";
                    //         let bytes = hex::decode(KEY).expect("cannot decode key");
                    //         println!("key: {}", hex::encode(&bytes));
                    //         let value = api.storage()
                    //                        .mvm()
                    //                        .vm_storage(bytes, None)
                    //                        .await
                    //                        .expect("cannot get storage by key");
                    //         println!("pontem.move STORAGE::28e050611b6cb9358721c8...: {:?}",
                    //                  value.map(|v| hex::encode(v)));
                    //     }
                    //     _ => unimplemented!("feature not supported for {} runtime", client),
                    // }
                }
                _ => unimplemented!("not supported option {}", c),
            },
            _ => unimplemented!("not supported option {}", b),
        },

        "tx" => {
            match b {
                "balances" => {
                    match c {
                        "transfer" => {
                            // TODO: ensure that call.args.len() == 2;

                            // TODO: ensure that call.signer.is_some
                            // let signer = call.signer.as_ref().expect("signer should be");

                            // TODO: eval dest, amount & signer


                            same_for_all!(client, |api| {
                                println!("TRANSFER: {}({:?})", call.method, call.args);

                                // send transfer

                                use client::sp_core::Pair;
                                // // let pair = subxt::sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
                                // let pair: subxt::sp_core::sr25519::Pair = keys::pair_from_str("//Alice").unwrap();

                                // let dest: client::sp_runtime::AccountId32 = pair.clone().public().into();
                                // let dest = subxt::sp_runtime::MultiAddress::from(dest);

                                let signer = if let Some(signer) = call.signer.as_ref() {
                                    keys::signer_from_str(signer).unwrap()
                                } else {
                                    // return Err(TODO: "signer should be");
                                    panic!("signer account should be");
                                };

                                let dest = if let Some(dest) = call.args.get(0) {
                                    keys::id_from_str(dest).unwrap()
                                } else {
                                    // return Err(TODO: "signer should be");
                                    panic!("destination account should be");
                                };

                                let result = api.tx()
                                                .balances()
                                                .transfer(dest.into(), 10_000)
                                                .sign_and_submit_then_watch(&signer)
                                                // default check:
                                                .and_then(|result| async {
                                                    println!("RESULT: {} :: {}", result.block, result.extrinsic);
                                                    // println!("RESULT.EVENTS: {:#?}", result.events);

                                                    // check result
                                                    // event:
                                                    let e = result.find_event::<api_current::balances::events::Transfer>();
                                                    // TODO: Error:
                                                    // - Ok(None) -> "not found"
                                                    // - Err(CodecError) -> Err(CodecError)
                                                    //
                                                    // Ok(e) -> put into ctx
                                                    println!("EVENT: {:#?}", e);

                                                    // const ID: &str = "Balances::Transfer";

                                                    if let Some(event) = e? {
                                                        println!("Balance transfer success: value: {:?}", event.2);

                                                        // crate::eval::add_events_to_context(ctx, &result.events)?;

                                                        let api_current::balances::events::Transfer(from, to, amount) = event;
                                                        let from = format!("{}", from.to_ss58check());
                                                        let to = format!("{}", to.to_ss58check());
                                                    } else {
                                                        println!("Failed to find Balances::Transfer Event");
                                                        // TODO: should fail the step
                                                    }
                                                    Ok(result)
                                                })
                                                // .then(check_ext_result)
                                                .await?;
                                // TODO: continue
                                println!("COMPLETE: {} :: {}", result.block, result.extrinsic);
                            });
                        }
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}


use client::keys;


// async fn check_ext_result<T>(result: Result<subxt::ExtrinsicSuccess<T>, subxt::Error>)
//                              -> Result<subxt::ExtrinsicSuccess<T>, BoxErr>
//     where T: subxt::Config {
//     // result.map();

//     Ok(result)
// }
