//! Standalone functions for engine specific rpc type conversions
pub mod payload;
pub use payload::{block_to_payload_v1, try_into_sealed_block, try_payload_v1_to_block};

/// [kasplex]: Kasplex-specific ExecutionPayload extensions
#[cfg(feature = "kasplex")]
pub mod kasplex;
#[cfg(feature = "kasplex")]
pub use kasplex::{block_to_kasplex_payload, kasplex_payload_to_block, KasplexExecutionPayload};
