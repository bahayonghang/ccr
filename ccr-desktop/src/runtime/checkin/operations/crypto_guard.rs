//! Existing encrypted rows must never be paired with an initialized replacement key.
use super::*;
use ccr_checkin::core::crypto::CryptoManager;

impl Storage {
    pub(super) fn require_existing_crypto(&self) -> Result<(), OperationError> {
        CryptoManager::load_existing(&self.crypto)
            .map(|_| ())
            .map_err(|_| OperationError::CryptoKeyUnavailable)
    }

    /// Empty captured storage keeps the existing first-use initialization path.
    pub(super) fn guard_crypto_initialization(&self) -> Result<(), OperationError> {
        if !self
            .accounts()
            .load_all()
            .map_err(|_| OperationError::Storage)?
            .is_empty()
        {
            self.require_existing_crypto()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
