//! [kasplex]: Kasplex-specific EVM execution logic

#[cfg(feature = "kasplex")]
use reth_chainspec::ChainSpec;
#[cfg(feature = "kasplex")]
use reth_primitives::{keccak256, Address};

/// [kasplex]: Calculate the treasury address based on chain ID
///
/// The treasury address is calculated as: `0x{ChainID}{padding zeros}{10001}`
/// where the total length is 40 hex characters (20 bytes).
#[cfg(feature = "kasplex")]
pub fn get_treasury_address(chain_spec: &ChainSpec) -> Address {
    let chain_id = chain_spec.chain.id();
    let chain_id_str = chain_id.to_string();
    let suffix = "10001";
    
    // Calculate padding: 40 hex chars (20 bytes) - chain_id length - suffix length
    let padding_len = 40usize.saturating_sub(chain_id_str.len() + suffix.len());
    let padding = "0".repeat(padding_len);
    
    let address_str = format!("0x{}{}{}", chain_id_str, padding, suffix);
    // Parse hex string to address using alloy_primitives::hex
    if let Ok(addr) = alloy_primitives::hex::FromHex::from_hex(&address_str) {
        addr
    } else {
        // Fallback: use a simple hash-based address if parsing fails
        let hash = keccak256(format!("kasplex_treasury_{}", chain_id));
        Address::from_slice(&hash[..20])
    }
}

#[cfg(test)]
#[cfg(feature = "kasplex")]
mod tests {
    use super::*;
    use reth_chainspec::{ChainSpec, KASPLEX_INTERNAL_L2};

    #[test]
    fn test_treasury_address() {
        let chain_spec = KASPLEX_INTERNAL_L2.clone();
        let treasury = get_treasury_address(&chain_spec);
        // Treasury address should be deterministic based on chain ID
        assert_ne!(treasury, Address::ZERO);
    }
}

