//! [kasplex]: Kasplex Auth RPC API interface

#[cfg(feature = "kasplex")]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
#[cfg(feature = "kasplex")]
use reth_primitives::{Bytes, U256};
#[cfg(feature = "kasplex")]
use reth_rpc_types::txpool::TxpoolContent as TxPoolContent;

/// [kasplex]: Kasplex Auth RPC API namespace for authenticated APIs
#[cfg(feature = "kasplex")]
#[cfg_attr(not(feature = "client"), rpc(server, namespace = "kasplexAuth"))]
#[cfg_attr(feature = "client", rpc(server, client, namespace = "kasplexAuth"))]
pub trait KasplexAuthApi {
    /// Returns the content of the transaction pool.
    #[method(name = "txPoolContent")]
    async fn tx_pool_content(&self) -> RpcResult<TxPoolContent>;

    /// Returns the content of the transaction pool with minimum tip.
    #[method(name = "txPoolContentWithMinTip")]
    async fn tx_pool_content_with_min_tip(&self, min_tip: U256) -> RpcResult<TxPoolContent>;

    /// Sends a raw transaction.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<reth_primitives::B256>;

    /// Sends multiple raw transactions.
    #[method(name = "sendRawTransactions")]
    async fn send_raw_transactions(&self, txs: Vec<Bytes>) -> RpcResult<Vec<reth_primitives::B256>>;
}

