use crate::conditions::cfg::ConditionsCfg;
use crate::error::TestError;
use async_trait::async_trait;
use serde::{Serialize, Deserialize, de::DeserializeOwned};


pub mod proc;


#[async_trait(?Send)]
pub trait Task<Cfg: TaskCfg> {
    // type Conditions = Cfg::Conditions;
    async fn run(self, cfg: Cfg, conds: ConditionsCfg<Cfg::Conditions>) -> Result<(), TestError>;
    // run<S>(cfg: ProcRunCfg<S>,
    // conds: ConditionsCfg)
}

pub trait TaskCfg: Serialize + DeserializeOwned {
    // type Conditions: TaskConditions;
    type Conditions: Serialize + DeserializeOwned;
}

pub trait TaskConditions: Serialize + DeserializeOwned {}
