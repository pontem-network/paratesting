use crate::conditions::cfg;
use crate::error::TestError;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};


pub mod proc;


#[async_trait(?Send)]
pub trait Task<Cfg: Config> {
    async fn run(self, cfg: Cfg, conds: cfg::Config<Cfg::Conditions>) -> Result<(), TestError>;
}

pub trait Config: Serialize + DeserializeOwned {
    type Conditions: Serialize + DeserializeOwned;
}
