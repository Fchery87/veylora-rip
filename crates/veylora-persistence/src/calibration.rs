//! Immutable calibration record storage, keyed by verified content hash. In memory only:
//! durable calibration persistence is not part of this task. No recipe promotion is performed.
use std::{collections::BTreeMap, error::Error, fmt};
use veylora_color::CalibrationRecord;

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    Conflict,
    InvalidHash,
    Serialization(String),
}
impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for StoreError {}
#[derive(Default)]
pub struct CalibrationStore {
    records: BTreeMap<String, CalibrationRecord>,
}
impl CalibrationStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn put(&mut self, record: CalibrationRecord) -> Result<(), StoreError> {
        if record.content_hash != record.compute_hash() {
            return Err(StoreError::InvalidHash);
        }
        let key = record.content_hash.to_string();
        if let Some(existing) = self.records.get(&key) {
            if existing != &record {
                return Err(StoreError::Conflict);
            }
            return Ok(());
        }
        self.records.insert(key, record);
        Ok(())
    }
    pub fn get(&self, hash: &str) -> Option<&CalibrationRecord> {
        self.records.get(hash)
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn export_json(&self, hash: &str) -> Result<Option<String>, StoreError> {
        self.get(hash)
            .map(|r| serde_json::to_string(r).map_err(|e| StoreError::Serialization(e.to_string())))
            .transpose()
    }
}
