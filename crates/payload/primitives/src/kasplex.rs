//! [kasplex]: Kasplex-specific payload types

#[cfg(feature = "kasplex")]
use crate::{validate_version_specific_fields, EngineApiMessageVersion, EngineObjectValidationError, PayloadAttributes, PayloadOrAttributes};
#[cfg(feature = "kasplex")]
use reth_chainspec::ChainSpec;
#[cfg(feature = "kasplex")]
use reth_primitives::{Address, B256, Bytes, U256};
#[cfg(feature = "kasplex")]
use reth_rpc_types::{engine::PayloadAttributes as EthPayloadAttributes, Withdrawal};
#[cfg(feature = "kasplex")]
use serde::{Deserialize, Serialize};

/// [kasplex]: BlockMetadata represents a `BlockMetadata` struct defined in protocol
#[cfg(feature = "kasplex")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Beneficiary address
    pub beneficiary: Address,
    /// Gas limit
    pub gas_limit: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Mix hash
    pub mix_hash: B256,
    /// Transaction list
    pub tx_list: Bytes,
    /// Extra data
    pub extra_data: Bytes,
}

/// [kasplex]: Kasplex-specific PayloadAttributes
///
/// Extends EthPayloadAttributes with Kasplex-specific fields:
/// - BaseFeePerGas: The base fee per gas for the block
/// - BlockMetadata: Block metadata including beneficiary, gas limit, etc.
#[cfg(feature = "kasplex")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KasplexPayloadAttributes {
    /// Inner Ethereum payload attributes
    #[serde(flatten)]
    pub payload_attributes: EthPayloadAttributes,
    /// [kasplex]: Base fee per gas
    pub base_fee_per_gas: U256,
    /// [kasplex]: Block metadata
    pub block_metadata: BlockMetadata,
}

#[cfg(feature = "kasplex")]
impl PayloadAttributes for KasplexPayloadAttributes {
    fn timestamp(&self) -> u64 {
        self.payload_attributes.timestamp
    }

    fn withdrawals(&self) -> Option<&Vec<Withdrawal>> {
        self.payload_attributes.withdrawals.as_ref()
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.payload_attributes.parent_beacon_block_root
    }

    fn ensure_well_formed_attributes(
        &self,
        chain_spec: &ChainSpec,
        version: EngineApiMessageVersion,
    ) -> Result<(), EngineObjectValidationError> {
        // Validate base Ethereum payload attributes using PayloadOrAttributes wrapper
        validate_version_specific_fields(
            chain_spec,
            version,
            PayloadOrAttributes::PayloadAttributes(self),
        )?;

        // [kasplex]: Additional validation for Kasplex-specific fields
        if !chain_spec.is_kasplex() {
            return Err(EngineObjectValidationError::InvalidParams(
                "KasplexPayloadAttributes can only be used with Kasplex chains".to_string().into(),
            ));
        }

        // Validate base fee per gas is set
        if self.base_fee_per_gas == U256::ZERO {
            return Err(EngineObjectValidationError::InvalidParams(
                "BaseFeePerGas must be set for Kasplex payload attributes".to_string().into(),
            ));
        }

        Ok(())
    }
}

/// [kasplex]: Generates the payload id for Kasplex payload from the [`KasplexPayloadAttributes`].
///
/// Returns an 8-byte identifier by hashing the payload components with sha256 hash.
/// Includes TxListHash in the hash calculation.
#[cfg(feature = "kasplex")]
pub fn payload_id_kasplex(
    parent: &B256,
    attributes: &KasplexPayloadAttributes,
) -> reth_rpc_types::engine::PayloadId {
    use reth_primitives::keccak256;
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(parent.as_slice());
    hasher.update(&attributes.payload_attributes.timestamp.to_be_bytes()[..]);
    hasher.update(attributes.payload_attributes.prev_randao.as_slice());
    hasher.update(attributes.payload_attributes.suggested_fee_recipient.as_slice());
    if let Some(withdrawals) = &attributes.payload_attributes.withdrawals {
        let mut buf = Vec::new();
        use alloy_rlp::Encodable;
        withdrawals.encode(&mut buf);
        hasher.update(buf);
    }

    if let Some(parent_beacon_block) = attributes.payload_attributes.parent_beacon_block_root {
        hasher.update(parent_beacon_block);
    }

    // [kasplex]: Include base fee per gas
    let base_fee_bytes: [u8; 32] = attributes.base_fee_per_gas.to_be_bytes();
    hasher.update(&base_fee_bytes);

    // [kasplex]: Include TxListHash (hash of tx_list)
    let tx_list_hash = keccak256(&attributes.block_metadata.tx_list);
    hasher.update(tx_list_hash.as_slice());

    // [kasplex]: Include block metadata fields
    hasher.update(attributes.block_metadata.beneficiary.as_slice());
    hasher.update(&attributes.block_metadata.gas_limit.to_be_bytes()[..]);
    hasher.update(&attributes.block_metadata.timestamp.to_be_bytes()[..]);
    hasher.update(attributes.block_metadata.mix_hash.as_slice());
    hasher.update(&attributes.block_metadata.extra_data);

    let out = hasher.finalize();
    reth_rpc_types::engine::PayloadId::new(out.as_slice()[..8].try_into().expect("sufficient length"))
}

