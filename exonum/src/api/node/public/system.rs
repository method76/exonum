//! Public system API.

use api::{ServiceApiScope, ServiceApiState};
use blockchain::{Schema, SharedNodeState};
use helpers::user_agent;

/// Information about the current state of the node memory pool.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MemPoolInfo {
    /// Total number of uncommitted transactions.
    pub size: u64,
}

/// Information about the amount of peers connected to the node.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PeersAmount {
    /// Amount of connected peers.
    pub amount: usize,
}

/// Information about whether the node is connected to other peers.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ConnectivityStatus {
    /// The node has no connected peers.
    NotConnected,
    /// The node has connected peers. Amount of connected peers is stored within this variant.
    Connected(PeersAmount),
}

/// Information about whether it is possible to achieve the consensus between
/// validators in the current state.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ConsensusStatus {
    /// Consensus disabled on this node.
    Disabled,
    /// Consensus enabled on this node.
    Enabled,
    /// Consensus enabled and the node has enough connected peers.
    Active,
}

/// Information about whether the node is connected to other peers and
/// its consensus status.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct HealthCheckInfo {
    /// Consensus status.
    pub consensus_status: ConsensusStatus,
    /// Connectivity status.
    pub connectivity: ConnectivityStatus,
}

/// Public system API.
#[derive(Clone, Debug)]
pub struct SystemApi {
    shared_api_state: SharedNodeState,
}

impl SystemApi {
    /// Creates a new `public::SystemApi` instance.
    pub fn new(shared_api_state: SharedNodeState) -> Self {
        Self { shared_api_state }
    }

    fn handle_mempool_info(self, name: &'static str, api_scope: &mut ServiceApiScope) -> Self {
        api_scope.endpoint(name, move |state: &ServiceApiState, _query: ()| {
            let snapshot = state.snapshot();
            let schema = Schema::new(&snapshot);
            Ok(MemPoolInfo {
                size: schema.transactions_pool_len(),
            })
        });
        self
    }

    fn handle_user_agent_info(self, name: &'static str, api_scope: &mut ServiceApiScope) -> Self {
        api_scope.endpoint(name, move |_state: &ServiceApiState, _query: ()| {
            Ok(user_agent::get())
        });
        self
    }

    fn handle_healthcheck_info(self, name: &'static str, api_scope: &mut ServiceApiScope) -> Self {
        let self_ = self.clone();
        api_scope.endpoint(name, move |_state: &ServiceApiState, _query: ()| {
            Ok(HealthCheckInfo {
                consensus_status: self.get_consensus_status(),
                connectivity: self.get_connectivity_status(),
            })
        });
        self_
    }

    fn get_connectivity_status(&self) -> ConnectivityStatus {
        let in_conn = self.shared_api_state.incoming_connections().len();
        let out_conn = self.shared_api_state.outgoing_connections().len();

        if in_conn == 0 && out_conn == 0 {
            ConnectivityStatus::NotConnected
        } else {
            ConnectivityStatus::Connected(PeersAmount {
                amount: in_conn + out_conn,
            })
        }
    }

    fn get_consensus_status(&self) -> ConsensusStatus {
        if self.shared_api_state.is_enabled() {
            if self.shared_api_state.consensus_status() {
                ConsensusStatus::Active
            } else {
                ConsensusStatus::Enabled
            }
        } else {
            ConsensusStatus::Disabled
        }
    }

    /// Adds public system API endpoints to the corresponding scope.
    pub fn wire(self, api_scope: &mut ServiceApiScope) -> &mut ServiceApiScope {
        self.handle_mempool_info("v1/mempool", api_scope)
            .handle_healthcheck_info("v1/healthcheck", api_scope)
            .handle_user_agent_info("v1/user_agent", api_scope);
        api_scope
    }
}
