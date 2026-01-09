use archivum_core::state::metadata_storage::{ LocalMetadataStorage };
use gloo_storage::{ LocalStorage, Storage };

pub struct LocalstorageMetadataStorage;

impl LocalMetadataStorage for LocalstorageMetadataStorage {
    type Error = LocalstorageMetadataStorageError;

    async fn save_metadata(&mut self, json: &str) -> Result<(), Self::Error> {
        LocalStorage::set("archivum-drive-repo", json).map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(e.to_string())
        )
    }

    async fn load_metadata(&self) -> Result<String, Self::Error> {
        LocalStorage::get("archivum-drive-repo").map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(e.to_string())
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LocalstorageMetadataStorageError {
    #[error("Localstorage error: {0}")] LocalstorageError(String),
}
