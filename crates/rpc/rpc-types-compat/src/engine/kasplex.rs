//! [kasplex]: Kasplex-specific ExecutionPayload extensions

#[cfg(feature = "kasplex")]
use reth_primitives::{B256, SealedBlock, TransactionSigned};
#[cfg(feature = "kasplex")]
use reth_rpc_types::engine::ExecutionPayload;

/// [kasplex]: Extended ExecutionPayload with Kasplex-specific fields
///
/// This wrapper extends ExecutionPayload with Numbers field for transaction submission block numbers.
/// Since ExecutionPayload is from an external crate, we wrap it to add Kasplex-specific fields.
#[cfg(feature = "kasplex")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KasplexExecutionPayload {
    /// Inner execution payload
    pub payload: ExecutionPayload,
    /// [kasplex]: Transaction submission block numbers
    pub numbers: Option<Vec<u64>>,
    /// [kasplex]: Transaction hash
    pub tx_hash: Option<B256>,
    /// [kasplex]: Withdrawals hash
    pub withdrawals_hash: Option<B256>,
}

#[cfg(feature = "kasplex")]
impl KasplexExecutionPayload {
    /// Create a new KasplexExecutionPayload from an ExecutionPayload
    pub fn new(payload: ExecutionPayload) -> Self {
        Self {
            payload,
            numbers: None,
            tx_hash: None,
            withdrawals_hash: None,
        }
    }

    /// Set numbers field
    pub fn with_numbers(mut self, numbers: Vec<u64>) -> Self {
        self.numbers = Some(numbers);
        self
    }

    /// Set transaction hash
    pub fn with_tx_hash(mut self, tx_hash: B256) -> Self {
        self.tx_hash = Some(tx_hash);
        self
    }

    /// Set withdrawals hash
    pub fn with_withdrawals_hash(mut self, withdrawals_hash: B256) -> Self {
        self.withdrawals_hash = Some(withdrawals_hash);
        self
    }
}

/// [kasplex]: Convert SealedBlock to KasplexExecutionPayload
///
/// Extracts Numbers from the block's numbers field and includes them in the payload.
#[cfg(feature = "kasplex")]
pub fn block_to_kasplex_payload(value: SealedBlock) -> KasplexExecutionPayload {
    use crate::engine::payload::block_to_payload;

    let (payload, _) = block_to_payload(value.clone());
    
    // Extract numbers from block
    let numbers = value.numbers.clone();
    
    // Extract transaction hash from block
    let tx_hash = if !value.body.is_empty() {
        Some(value.body[0].hash())
    } else {
        None
    };

    // Extract withdrawals hash from header
    let withdrawals_hash = value.header.withdrawals_root;

    KasplexExecutionPayload::new(payload)
        .with_numbers(numbers.unwrap_or_default())
        .with_tx_hash(tx_hash.unwrap_or_default())
        .with_withdrawals_hash(withdrawals_hash.unwrap_or_default())
}

/// [kasplex]: Convert KasplexExecutionPayload to Block
///
/// Sets transaction numbers from the Numbers field if available.
#[cfg(feature = "kasplex")]
pub fn kasplex_payload_to_block(
    payload: KasplexExecutionPayload,
    parent_beacon_block_root: Option<B256>,
) -> Result<reth_primitives::Block, reth_rpc_types::engine::PayloadError> {
    use crate::engine::payload::try_into_block;

    // Convert payload to block
    let mut block = try_into_block(payload.payload, parent_beacon_block_root)?;

    // Set transaction numbers if available
    if let Some(numbers) = payload.numbers {
        if numbers.len() == block.body.len() {
            for (tx, number) in block.body.iter_mut().zip(numbers.iter()) {
                #[cfg(feature = "kasplex")]
                {
                    tx.number = Some(*number);
                }
            }
        }
    }

    // Set block numbers field from transaction numbers
    #[cfg(feature = "kasplex")]
    {
        let numbers: Vec<u64> = block.body.iter()
            .filter_map(|tx| tx.number)
            .collect();
        if !numbers.is_empty() {
            block.numbers = Some(numbers);
        }
    }

    Ok(block)
}

