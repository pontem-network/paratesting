#[macro_use]
extern crate log;

pub extern crate subxt;
pub extern crate sp_keyring;

use std::fmt;
use serde::{Serialize, Deserialize};

pub use subxt::bitvec;
pub use subxt::codec;
pub use subxt::sp_arithmetic;
pub use subxt::sp_core;
pub use subxt::sp_runtime;

use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, PairSigner};
use subxt::EventsDecoder;
use subxt::ExtrinsicSuccess;
use subxt::ExtrinsicExtraData;
use subxt::sp_core::crypto::Ss58Codec;


pub mod keys;


pub type BoxErr = Box<dyn std::error::Error>;
pub type BoxRes<T, E = BoxErr> = Result<T, E>;


#[cfg(feature = "runtime-polkadot")]
#[subxt::subxt(runtime_metadata_path = "../metadata/polkadot_metadata.scale",
               generated_type_derives = "Clone, Debug")]
pub mod polkadot {}


#[cfg(feature = "runtime-pontem")]
// #[cfg_attr(…, path = …)]
// #[subxt::subxt(runtime_metadata_path = "../metadata/pontem-xcmp.scale")]
#[subxt::subxt(runtime_metadata_path = "../metadata/pontem.scale",
               generated_type_derives = "Clone, Debug")]
// #[path = "../gen/pontem.rs"]
pub mod pontem {
    // use subxt::codec::{Encode, Decode};
    // #[derive(Eq, PartialEq, Encode, Decode, Clone, Debug)]
    // #[subxt(substitute_type = "polkadot_parachain::primitives::Id")]
    // pub struct ParachainId(pub ::core::primitive::u32);
}
/// Fix for pontem meta - extra impl required by scale-codec.
#[cfg(feature = "runtime-pontem")]
const _: () = {
    use pontem::runtime_types::polkadot_parachain::primitives::Id;
    impl PartialEq for Id {
        fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
    }
    impl Eq for Id {}
    impl PartialOrd for Id {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }
    impl Ord for Id {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    }
};

#[cfg(feature = "runtime-rococo")]
#[subxt::subxt(runtime_metadata_path = "../metadata/rococo-local.scale",
               generated_type_derives = "Clone, Debug")]
pub mod rococo {}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedRuntime {
    /// Generic polkadot runtime
    /// TODO: describe version here
    #[serde(alias = "polka")]
    #[cfg(feature = "runtime-polkadot")]
    Polkadot,

    /// TODO: describe version here
    #[cfg(feature = "runtime-pontem")]
    Pontem,

    /// TODO: describe version here
    #[cfg(feature = "runtime-rococo")]
    Rococo,
}

impl SupportedRuntime {
    pub fn name(&self) -> &'static str {
        use SupportedRuntime::*;
        match self {
            #[cfg(feature = "runtime-pontem")]
            Pontem => "Pontem",
            #[cfg(feature = "runtime-rococo")]
            Rococo => "Rococo",
            #[cfg(feature = "runtime-polkadot")]
            Polkadot => "Polkadot",
        }
    }
}

impl From<&'_ NodeRuntimeApi> for SupportedRuntime {
    fn from(c: &NodeRuntimeApi) -> Self {
        match c {
            #[cfg(feature = "runtime-pontem")]
            NodeRuntimeApi::Pontem(_) => SupportedRuntime::Pontem,
            #[cfg(feature = "runtime-rococo")]
            NodeRuntimeApi::Rococo(_) => SupportedRuntime::Rococo,
            #[cfg(feature = "runtime-polkadot")]
            NodeRuntimeApi::Polkadot(_) => SupportedRuntime::Polkadot,
        }
    }
}

impl fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.name()) }
}


pub trait RuntimeSupport {
    type Cfg: subxt::Config;
    type Api: subxt::Config + subxt::ExtrinsicExtraData<Self::Cfg>;
}


pub enum NodeRuntimeApi {
    #[cfg(feature = "runtime-pontem")]
    Pontem(pontem::RuntimeApi<pontem::DefaultConfig>),
    #[cfg(feature = "runtime-polkadot")]
    Polkadot(polkadot::RuntimeApi<polkadot::DefaultConfig>),
    #[cfg(feature = "runtime-rococo")]
    Rococo(rococo::RuntimeApi<rococo::DefaultConfig>),
}

impl NodeRuntimeApi {
    pub async fn new<S: Into<String>>(url: S,
                                      runtime: SupportedRuntime)
                                      -> Result<Self, subxt::Error> {
        let client = match runtime {
            #[cfg(feature = "runtime-pontem")]
            SupportedRuntime::Pontem => {
                NodeRuntimeApi::Pontem(ClientBuilder::new().set_url(url)
                                                           .build::<pontem::DefaultConfig>()
                                                           .await?
                                                           .to_runtime_api::<pontem::RuntimeApi<_>>())
            }
            #[cfg(feature = "runtime-rococo")]
            SupportedRuntime::Rococo => {
                NodeRuntimeApi::Rococo(ClientBuilder::new().set_url(url)
                                                           .build::<rococo::DefaultConfig>()
                                                           .await?
                                                           .to_runtime_api::<rococo::RuntimeApi<_>>())
            }
            #[cfg(feature = "runtime-polkadot")]
            SupportedRuntime::Polkadot => {
                NodeRuntimeApi::Polkadot(ClientBuilder::new().set_url(url)
                                                             .build::<polkadot::DefaultConfig>()
                                                             .await?
                                                             .to_runtime_api::<polkadot::RuntimeApi<_>>())
            }
        };

        Ok(client)
    }
}


impl fmt::Display for NodeRuntimeApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rt: SupportedRuntime = self.into();
        write!(f, "{}", rt.name())
    }
}


// pub async fn call_to_node(client: &NodeRuntimeApi,
//                           call: &CallData,
//                           /* ctx: &mut Ctx */)
//                           -> Result<(), BoxErr> {
//     let (a, b, c) = {
//         let ref abc = call.method.split('.').collect::<Vec<&str>>()[..3];
//         (abc[0], abc[1], abc[2])
//     };

//     // TODO: // eval_with_context(method, ctx)?;

//     macro_rules! same_for_all {
// 		// `()` indicates that the macro takes no argument.
// 		($client:ident, | | $block:expr) => { same_for_all!($client, || $block) };
// 		($client:ident, || $block:expr) => {
// 			match $client {
// 				NodeRuntimeApi::Pontem(..) => $block,
// 				NodeRuntimeApi::Rococo(..) => $block,
// 				NodeRuntimeApi::Polkadot(..) => $block,
// 			}
// 		};
// 		($client:ident, |$($api:ident),+| $block:expr) => {
// 			match $client {
// 				NodeRuntimeApi::Pontem($($api)*, ..) => $block,
// 				NodeRuntimeApi::Rococo($($api)*, ..) => $block,
// 				NodeRuntimeApi::Polkadot($($api)*, ..) => $block,
// 			}
// 		};
// 	}

//     match a {
//         "tx" => {
//             match b {
//                 "balances" => {
//                     match c {
//                         "transfer" => {
//                             println!("TRANSFER: {}({:?})", call.method, call.args);

//                             // TODO: ensure that call.args.len() == 2;

//                             // TODO: ensure that call.signer.is_some
//                             // let signer = call.signer.as_ref().expect("signer should be");

//                             // TODO: eval dest, amount & signer

//                             // :subxt::sp_runtime::MultiAddress<subxt::sp_runtime::AccountId32, ()>

//                             // let dest: subxt::sp_runtime::MultiAddress<subxt::sp_runtime::AccountId32, ()> =
//                             // 	AccountKeyring::Bob.to_account_id().into();

//                             same_for_all!(client, |api| {
//                                 // let dest = AccountKeyring::Bob.to_account_id().into();
//                                 // println!("dest: {:?}", dest);

//                                 // let signer = PairSigner::new(AccountKeyring::Alice.pair());
//                                 // // let signer: subxt::PairSigner<
//                                 // //                               crate::api::pontem::api::DefaultConfig,
//                                 // //                               sp_keyring::sr25519::sr25519::Pair,
//                                 // // > = crate::eval::signer_from_str(&signer)?;
//                                 // let result = api.tx()
//                                 //                 .balances()
//                                 //                 .transfer(dest, 10_000)
//                                 //                 .sign_and_submit_then_watch(&signer)
//                                 //                 .await?;
//                                 // println!("RESULT: {} :: {}", result.block, result.extrinsic);

//                                 // // check result
//                                 // // event:
//                                 // let e = result.find_event::<pontem::balances::events::Transfer>();
//                                 // println!("EVENT: {:#?}", e);

//                                 // const ID: &str = "Balances::Transfer";

//                                 // if let Some(event) = e? {
//                                 // 	println!("Balance transfer success: value: {:?}", event.2);

//                                 // 	// crate::eval::add_events_to_context(ctx, &result.events)?;

//                                 // 	let pontem::balances::events::Transfer(from, to, amount) = event;
//                                 // // let from = format!("{}", from.to_ss58check());
//                                 // // let to = format!("{}", to.to_ss58check());
//                                 // // use evalexpr::*;
//                                 // // ctx.set_value(format!("{}.from", ID), Value::from(from))?;
//                                 // // ctx.set_value(format!("{}.to", ID), Value::from(to))?;
//                                 // // ctx.set_value(format!("{}.amount", ID), Value::from(amount as i64))?;
//                                 // } else {
//                                 // 	println!("Failed to find Balances::Transfer Event");
//                                 // 	// TODO: should fail the step
//                                 // }
//                             });
//                         }
//                         _ => unimplemented!(),
//                     }
//                 }
//                 _ => unimplemented!(),
//             }
//         }
//         _ => unimplemented!(),
//     }
//     Ok(())
// }
