//! [kasplex]: Transaction hash to block number mapping module
//!
//! This module provides thread-safe operations for managing transaction hash to block number
//! mappings, which are used in Kasplex networks to track when transactions were submitted.

#[cfg(feature = "kasplex")]
use crate::TxHash;
#[cfg(feature = "kasplex")]
use parking_lot::RwLock;
#[cfg(feature = "kasplex")]
use std::collections::HashMap;
#[cfg(feature = "kasplex")]
use std::sync::atomic::{AtomicI32, Ordering};

/// [kasplex]: Number of blocks to retain unexecuted transactions
pub const UNEXECUTED_TX_RETENTION_BLOCKS: u64 = 100;

/// [kasplex]: Thread-safe transaction hash to block number mapping
#[cfg(feature = "kasplex")]
pub struct TxMapping {
    /// Mapping from transaction hash to block number
    mapping: RwLock<HashMap<TxHash, u64>>,
    /// Deleted transactions set
    deleted: RwLock<HashMap<TxHash, ()>>,
    /// Reorg counter
    reorg_num: AtomicI32,
}

#[cfg(feature = "kasplex")]
impl Default for TxMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "kasplex")]
impl TxMapping {
    /// Create a new transaction mapping instance
    pub fn new() -> Self {
        Self {
            mapping: RwLock::new(HashMap::new()),
            deleted: RwLock::new(HashMap::new()),
            reorg_num: AtomicI32::new(0),
        }
    }

    /// Set the entire mapping dictionary
    pub fn set_mapping(&self, dic: HashMap<TxHash, u64>) {
        let mut mapping = self.mapping.write();
        *mapping = dic;
    }

    /// Get the block number for a transaction hash and remove it from the mapping
    ///
    /// Returns `None` if the transaction hash is not found
    pub fn get_tx_number(&self, hash: TxHash) -> Option<u64> {
        let mut mapping = self.mapping.write();
        mapping.remove(&hash)
    }

    /// Get the block number for a transaction hash without removing it
    ///
    /// Returns `None` if the transaction hash is not found
    pub fn peek_tx_number(&self, hash: TxHash) -> Option<u64> {
        let mapping = self.mapping.read();
        mapping.get(&hash).copied()
    }

    /// Insert a transaction hash to block number mapping
    pub fn insert(&self, hash: TxHash, number: u64) {
        let mut mapping = self.mapping.write();
        mapping.insert(hash, number);
    }

    /// Mark that a pool reorg has occurred
    pub fn mark_pool_reorged(&self) {
        self.reorg_num.fetch_sub(1, Ordering::SeqCst);
    }

    /// Mark the start of a pool reorg
    pub fn mark_start_pool_reorg(&self) {
        self.reorg_num.fetch_add(1, Ordering::SeqCst);
    }

    /// Clear the pool reorg state
    pub fn clear_pool_reorged(&self) {
        self.reorg_num.store(0, Ordering::SeqCst);
    }

    /// Get the current reorg number
    pub fn get_reorg_num(&self) -> i32 {
        self.reorg_num.load(Ordering::SeqCst)
    }

    /// Check if reorg is complete
    ///
    /// Returns `true` if no reorg is in progress
    pub fn is_reorg_complete(&self) -> bool {
        if self.get_reorg_num() > 0 {
            return false;
        }
        // Small delay to ensure consistency
        std::thread::sleep(std::time::Duration::from_micros(50));
        self.get_reorg_num() <= 0
    }

    /// Add a transaction hash to the deleted transactions set
    pub fn add_deleted_transaction(&self, hash: TxHash) {
        let mut deleted = self.deleted.write();
        deleted.insert(hash, ());
    }

    /// Check if a transaction hash is in the deleted transactions set
    pub fn is_deleted_transaction(&self, hash: TxHash) -> bool {
        let deleted = self.deleted.read();
        deleted.contains_key(&hash)
    }

    /// Clear all deleted transactions
    pub fn clear_deleted_transaction(&self) {
        let mut deleted = self.deleted.write();
        deleted.clear();
        self.clear_pool_reorged();
    }

    /// Remove a transaction hash from the mapping
    pub fn remove(&self, hash: TxHash) -> Option<u64> {
        let mut mapping = self.mapping.write();
        mapping.remove(&hash)
    }

    /// Check if a transaction hash exists in the mapping
    pub fn contains_key(&self, hash: TxHash) -> bool {
        let mapping = self.mapping.read();
        mapping.contains_key(&hash)
    }

    /// Get the number of transactions in the mapping
    pub fn len(&self) -> usize {
        let mapping = self.mapping.read();
        mapping.len()
    }

    /// Check if the mapping is empty
    pub fn is_empty(&self) -> bool {
        let mapping = self.mapping.read();
        mapping.is_empty()
    }

    /// Clear all mappings
    pub fn clear(&self) {
        let mut mapping = self.mapping.write();
        mapping.clear();
    }
}

#[cfg(test)]
#[cfg(feature = "kasplex")]
mod tests {
    use super::*;
    use crate::B256;

    #[test]
    fn test_tx_mapping_basic() {
        let mapping = TxMapping::new();
        let hash = B256::random();

        // Initially empty
        assert!(mapping.is_empty());
        assert_eq!(mapping.get_tx_number(hash), None);

        // Insert and retrieve
        mapping.insert(hash, 100);
        assert_eq!(mapping.peek_tx_number(hash), Some(100));
        assert_eq!(mapping.get_tx_number(hash), Some(100));
        assert_eq!(mapping.get_tx_number(hash), None); // Should be removed after get
    }

    #[test]
    fn test_reorg_operations() {
        let mapping = TxMapping::new();

        assert_eq!(mapping.get_reorg_num(), 0);
        assert!(mapping.is_reorg_complete());

        mapping.mark_start_pool_reorg();
        assert_eq!(mapping.get_reorg_num(), 1);
        assert!(!mapping.is_reorg_complete());

        mapping.mark_pool_reorged();
        assert_eq!(mapping.get_reorg_num(), 0);
        assert!(mapping.is_reorg_complete());

        mapping.clear_pool_reorged();
        assert_eq!(mapping.get_reorg_num(), 0);
    }

    #[test]
    fn test_deleted_transactions() {
        let mapping = TxMapping::new();
        let hash = B256::random();

        assert!(!mapping.is_deleted_transaction(hash));

        mapping.add_deleted_transaction(hash);
        assert!(mapping.is_deleted_transaction(hash));

        mapping.clear_deleted_transaction();
        assert!(!mapping.is_deleted_transaction(hash));
    }
}

