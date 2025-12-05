//! [kasplex]: Kasplex Auth RPC API implementation

#[cfg(feature = "kasplex")]
use crate::{
    eth::EthApi,
    result::ToRpcResult,
};
#[cfg(feature = "kasplex")]
use async_trait::async_trait;
#[cfg(feature = "kasplex")]
use jsonrpsee::core::RpcResult as Result;
#[cfg(feature = "kasplex")]
use reth_primitives::{Address, Bytes, U256};
#[cfg(feature = "kasplex")]
use reth_provider::{BlockReaderIdExt, ChainSpecProvider};
#[cfg(feature = "kasplex")]
use reth_rpc_api::KasplexAuthApiServer;
#[cfg(feature = "kasplex")]
use reth_rpc_types::{Transaction, txpool::TxpoolContent};
#[cfg(feature = "kasplex")]
use reth_transaction_pool::{AllPoolTransactions, PoolTransaction, TransactionPool};
#[cfg(feature = "kasplex")]
use std::collections::BTreeMap;
#[cfg(feature = "kasplex")]
use tracing::trace;

/// [kasplex]: Kasplex Auth RPC API implementation
#[cfg(feature = "kasplex")]
#[async_trait]
impl<Provider, Pool, Network, EvmConfig> KasplexAuthApiServer for EthApi<Provider, Pool, Network, EvmConfig>
where
    Provider: BlockReaderIdExt + ChainSpecProvider + reth_provider::StateProviderFactory + reth_provider::EvmEnvProvider + 'static,
    Pool: TransactionPool + 'static,
    Network: reth_network_api::NetworkInfo + Send + Sync + 'static,
    EvmConfig: reth_evm::ConfigureEvm + 'static,
{
    /// Handler for: `kasplexAuth_txPoolContent`
    async fn tx_pool_content(&self) -> Result<TxpoolContent> {
        trace!(target: "rpc::kasplex_auth", "Serving kasplexAuth_txPoolContent");
        
        // Get transaction pool content similar to txpool_content
        #[inline]
        fn insert<T: PoolTransaction>(
            tx: &T,
            content: &mut BTreeMap<Address, BTreeMap<String, Transaction>>,
        ) {
            content.entry(tx.sender()).or_default().insert(
                tx.nonce().to_string(),
                reth_rpc_types_compat::transaction::from_recovered(tx.to_recovered_transaction()),
            );
        }

        let AllPoolTransactions { pending, queued } = self.pool().all_transactions();

        let mut content = TxpoolContent::default();
        for pending in pending {
            insert(&pending.transaction, &mut content.pending);
        }
        for queued in queued {
            insert(&queued.transaction, &mut content.queued);
        }

        Ok(content)
    }

    /// Handler for: `kasplexAuth_txPoolContentWithMinTip`
    async fn tx_pool_content_with_min_tip(&self, min_tip: U256) -> Result<TxpoolContent> {
        trace!(target: "rpc::kasplex_auth", ?min_tip, "Serving kasplexAuth_txPoolContentWithMinTip");
        
        // Get transaction pool content filtered by minimum tip
        #[inline]
        fn insert_with_min_tip<T: PoolTransaction>(
            tx: &T,
            min_tip: U256,
            base_fee: u64,
            content: &mut BTreeMap<Address, BTreeMap<String, Transaction>>,
        ) -> bool {
            // Calculate effective tip: min(max_fee_per_gas - base_fee, max_priority_fee_per_gas)
            let max_fee_per_gas = U256::from(tx.max_fee_per_gas());
            let base_fee_u256 = U256::from(base_fee);
            let effective_tip = if max_fee_per_gas > base_fee_u256 {
                let fee_cap_tip = max_fee_per_gas - base_fee_u256;
                if let Some(priority_fee) = tx.max_priority_fee_per_gas() {
                    U256::from(priority_fee).min(fee_cap_tip)
                } else {
                    fee_cap_tip
                }
            } else {
                U256::ZERO
            };

            // Only include transactions with tip >= min_tip
            if effective_tip >= min_tip {
                content.entry(tx.sender()).or_default().insert(
                    tx.nonce().to_string(),
                    reth_rpc_types_compat::transaction::from_recovered(tx.to_recovered_transaction()),
                );
                true
            } else {
                false
            }
        }

        // Get current base fee from the latest block
        let base_fee = self
            .provider()
            .header_by_number_or_tag(reth_primitives::BlockNumberOrTag::Latest)
            .ok()
            .flatten()
            .and_then(|h| h.base_fee_per_gas)
            .unwrap_or(0);

        let AllPoolTransactions { pending, queued } = self.pool().all_transactions();

        let mut content = TxpoolContent::default();
        for pending in pending {
            insert_with_min_tip(&pending.transaction, min_tip, base_fee, &mut content.pending);
        }
        for queued in queued {
            insert_with_min_tip(&queued.transaction, min_tip, base_fee, &mut content.queued);
        }

        Ok(content)
    }

    /// Handler for: `kasplexAuth_sendRawTransaction`
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<reth_primitives::B256> {
        trace!(target: "rpc::kasplex_auth", ?tx, "Serving kasplexAuth_sendRawTransaction");
        // Use the same implementation as eth_sendRawTransaction
        // The Number field will be set automatically in send_raw_transaction
        use crate::eth::EthTransactions;
        EthTransactions::send_raw_transaction(self, tx).await.to_rpc_result()
    }

    /// Handler for: `kasplexAuth_sendRawTransactions`
    async fn send_raw_transactions(&self, txs: Vec<Bytes>) -> Result<Vec<reth_primitives::B256>> {
        trace!(target: "rpc::kasplex_auth", ?txs, "Serving kasplexAuth_sendRawTransactions");
        // Send multiple transactions and return their hashes
        let mut hashes = Vec::new();
        for tx in txs {
            match self.send_raw_transaction(tx).await {
                Ok(hash) => hashes.push(hash),
                Err(e) => {
                    tracing::warn!(target: "rpc::kasplex_auth", error = ?e, "Failed to send transaction in batch");
                    // Continue processing other transactions
                }
            }
        }
        Ok(hashes)
    }
}

