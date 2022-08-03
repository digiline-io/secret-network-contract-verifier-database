use std::any::type_name;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use cosmwasm_std::{StdError, StdResult, Storage};

pub static ADMIN_KEY: &[u8] = b"admin";
pub static BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize)]
pub struct CompilationResult {
    pub(crate) code_id: u32,
    pub(crate) repo: String,
    pub(crate) commit_hash: String,
    pub(crate) method: String,
    pub(crate) verified: bool,
}

/// Returns StdResult<()> resulting from saving an item to storage
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage this item should go to
/// * `key` - a byte slice representing the key to access the stored item
/// * `value` - a reference to the item to store
pub fn save<T: Serialize>(storage: &mut dyn Storage, key: &[u8], value: &T) -> StdResult<()> {
    //storage.set(key, &Bincode2::serialize(value)?);
    //let x = bincode2::serialize(obj).map_err(|err| StdError::serialize_err(type_name::<T>(), err));
    let bin_data =
        bincode2::serialize(&value).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

/// Removes an item from storage
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage this item is in
/// * `key` - a byte slice representing the key that accesses the stored item
pub fn remove<S: Storage>(storage: &mut dyn Storage, key: &[u8]) {
    storage.remove(key);
}

/// Returns StdResult<T> from retrieving the item with the specified key.  Returns a
/// StdError::NotFound if there is no item with that key
///
/// # Arguments
///
/// * `storage` - a reference to the storage this item is in
/// * `key` - a byte slice representing the key that accesses the stored item
pub fn load<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<T> {
    let bin_data = storage
        .get(key)
        .ok_or_else(|| StdError::not_found(type_name::<T>()))?;
    bincode2::deserialize(&bin_data).map_err(|err| StdError::parse_err(type_name::<T>(), err))
}

/// Returns StdResult<Option<T>> from retrieving the item with the specified key.
/// Returns Ok(None) if there is no item with that key
///
/// # Arguments
///
/// * `storage` - a reference to the storage this item is in
/// * `key` - a byte slice representing the key that accesses the stored item
pub fn may_load<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => bincode2::deserialize(&value)
            .map_err(|err| StdError::parse_err(type_name::<T>(), err))
            .map(Some),
        None => Ok(None),
    }
}
