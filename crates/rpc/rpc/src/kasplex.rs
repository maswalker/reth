//! [kasplex]: Kasplex RPC API implementation

#[cfg(feature = "kasplex")]
use crate::eth::EthApi;
#[cfg(feature = "kasplex")]
use async_trait::async_trait;
#[cfg(feature = "kasplex")]
use jsonrpsee::core::RpcResult as Result;
#[cfg(feature = "kasplex")]
use reth_provider::{BlockReaderIdExt, ChainSpecProvider};
#[cfg(feature = "kasplex")]
use reth_rpc_api::KasplexApiServer;
#[cfg(feature = "kasplex")]
use tracing::trace;

/// [kasplex]: Kasplex RPC API implementation
#[cfg(feature = "kasplex")]
#[async_trait]
impl<Provider, Pool, Network, EvmConfig> KasplexApiServer for EthApi<Provider, Pool, Network, EvmConfig>
where
    Provider: BlockReaderIdExt + ChainSpecProvider + reth_provider::StateProviderFactory + reth_provider::EvmEnvProvider + 'static,
    Pool: reth_transaction_pool::TransactionPool + 'static,
    Network: reth_network_api::NetworkInfo + Send + Sync + 'static,
    EvmConfig: reth_evm::ConfigureEvm + 'static,
{
    /// Handler for: `kasplex_getSyncMode`
    async fn get_sync_mode(&self) -> Result<String> {
        trace!(target: "rpc::kasplex", "Serving kasplex_getSyncMode");
        // Check if the node is syncing
        use crate::eth::EthApiSpec;
        let is_syncing = EthApiSpec::is_syncing(self);
        // Return sync mode: "full" for full sync, "snap" for snap sync
        // For now, we return "full" as the default sync mode
        // In the future, this could be determined by checking the actual sync mode
        Ok(if is_syncing { "syncing".to_string() } else { "full".to_string() })
    }
}

