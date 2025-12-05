//! [kasplex]: Kasplex RPC API interface

#[cfg(feature = "kasplex")]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

/// [kasplex]: Kasplex RPC API namespace for public APIs
#[cfg(feature = "kasplex")]
#[cfg_attr(not(feature = "client"), rpc(server, namespace = "kasplex"))]
#[cfg_attr(feature = "client", rpc(server, client, namespace = "kasplex"))]
pub trait KasplexApi {
    /// Returns the sync mode of the node.
    #[method(name = "getSyncMode")]
    async fn get_sync_mode(&self) -> RpcResult<String>;
}

