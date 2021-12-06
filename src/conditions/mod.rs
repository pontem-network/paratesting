// use crate::format::suit::{ConditionsCfg, Conditions};
use crate::BoxErr;

pub mod cfg;
pub mod eval;


pub enum CheckResult {
    /// Success
    Success,
    Fail(BoxErr),
}


// pub struct ConditionsChecker<T> {
//     /// Describes how to test result of `target`.
//     cfg: ConditionsCfg,
//     /// This's result to check to meet conditions.
//     target: T,
// }

// impl<T> ConditionsChecker<T> {
//     pub fn new(cfg: ConditionsCfg, target: T) -> Self { Self { cfg, target } }
// }


// impl<T: client::subxt::Config> ConditionsChecker<client::subxt::ExtrinsicSuccess<T>> {}


// impl ConditionsChecker<subprocess::Popen> {}
