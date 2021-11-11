extern crate cel_interpreter;
extern crate cel_parser;
use crate::BoxErr;
use cel_interpreter::objects::CelType;
use cel_parser::Expression;
use cel_interpreter::Program;
use cel_interpreter::context::Context;

pub type Ctx = Context;


pub fn create_test_context() -> Result<Ctx, BoxErr> {
	// let mut ctx = {
	// 	use evalexpr::*;
	// 	context_map! {
	// 		"f" => Function::new(|argument| {
	// 			if let Ok(int) = argument.as_int() {
	// 				 Ok(Value::Int(int / 2))
	// 			} else if let Ok(float) = argument.as_float() {
	// 				 Ok(Value::Float(float / 2.0))
	// 			} else {
	// 				 Err(EvalexprError::expected_number(argument.clone()))
	// 			}
	// 		}),
	// 	}?
	// };

	// Ok(ctx)
	Ok(Context::default())
}


// subxt::ExtrinsicSuccess<api::pontem::api::DefaultConfig>
pub fn add_events_to_context(ctx: &mut Ctx, events: &[subxt::RawEvent]) -> Result<(), BoxErr> {
	// for e in events.iter() {
	// 	let ty = format!("{}::{}", e.pallet, e.variant);
	// 	ctx.add_variable(ty.clone(), CelType::Bool(true))?;
	// 	// ctx.add_variable(format!("{}.data", ty), CelType::Bytes(hex::encode(&e.data.0)))?;
	// 	ctx.add_variable(format!("{}.data", ty), CelType::Bytes(e.data.0))?;
	// }

	// let events = CelType::Map(events.iter()
	//                                 .map(|raw| Value::String(format!("{}::{}", raw.pallet, raw.variant)))
	//                                 .collect());
	// ctx.add_variable("events".to_owned(), events)?;

	Ok(())
}

/* TODO: impl this
pub fn del_events_from_context(ctx: &mut Ctx, events: &[subxt::RawEvent]) -> Result<(), EvalexprError> {
	let mut keys = Vec::new();
	keys.push("events".to_owned());
	for e in events.iter() {
		let ty = format!("{}::{}", e.pallet, e.variant);
		keys.push(ty.clone());
		keys.push(format!("{}.data", ty));
	}
	// TODO: `*ctx =` create new one without `keys`
	Ok(())
}
 */

// use sp_keyring::AccountKeyring;
// use subxt::PairSigner;
// use subxt::sp_runtime;
// use subxt::sp_core;
// use subxt::sp_core::sr25519;
// // use sp_keyring::sr25519::sr25519;
// use sp_runtime::traits::*;

// pub fn signer_from_str<T>(s: &str) -> Result<subxt::PairSigner<T, sr25519::Pair>, BoxErr>
// pub fn signer_from_str<T>(s: &str) -> Result<subxt::PairSigner<T, sp_keyring::sr25519::sr25519::Pair>, BoxErr>
// pub fn signer_from_str<T>(s: &str) -> Result<subxt::PairSigner<T, subxt::sp_core::sr25519::Pair>, BoxErr>
// 	where T: subxt::Config + subxt::ExtrinsicExtraData<T> {
// 	Ok(match s {
// 		"//Alice" | "Alice" => PairSigner::new(AccountKeyring::Alice.pair()),
// 		"//Bob" | "Bob" => PairSigner::new(AccountKeyring::Bob.pair()),
// 		"//Charlie" | "Charlie" => PairSigner::new(AccountKeyring::Charlie.pair()),
// 		"//Dave" | "Dave" => PairSigner::new(AccountKeyring::Dave.pair()),
// 		"//Eve" | "Eve" => PairSigner::new(AccountKeyring::Eve.pair()),
// 		"//Ferdie" | "Ferdie" => PairSigner::new(AccountKeyring::Ferdie.pair()),
// 		"//One" | "One" => PairSigner::new(AccountKeyring::One.pair()),
// 		"//Two" | "Two" => PairSigner::new(AccountKeyring::Two.pair()),
// 	})
// }
