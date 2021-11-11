

use std::collections::BTreeMap;

use crate::BoxErr;
use crate::api::NodeRuntimeApi;
use crate::format;
use format::suit::{ConnectCfg, NodeCfg};


pub type Nodes = BTreeMap<String, NodeCfg>;
pub type Clients = BTreeMap<String, NodeRuntimeApi>;


pub async fn create_clients_for_nodes(cfg: ConnectCfg) -> Result<(Nodes, Clients), BoxErr> {
	let mut nodes = BTreeMap::new();
	let mut clients = BTreeMap::new();

	for node in cfg.nodes.into_iter() {
		clients.insert(node.name.to_owned(), NodeRuntimeApi::new(&node).await?);
		nodes.insert(node.name.to_owned(), node.clone());
	}

	Ok((nodes, clients))
}
