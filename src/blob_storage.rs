use archivum_core::{ local_blob_storage::LocalBlobStore, blob::BlobId };
use crate::db::get_db;
use rexie::TransactionMode;
// use wasm_bindgen::JsValue; // Removed unused import

pub struct IndexedDbBlobStorage;

impl LocalBlobStore for IndexedDbBlobStorage {
    type Error = IndexedDbBlobStorageError;

    async fn store_blob(&mut self, data: &[u8], id: &BlobId) -> Result<(), Self::Error> {
        let db = get_db().await.map_err(|e|
            IndexedDbBlobStorageError::IndexedDbError(format!("{:?}", e))
        )?;
        let transaction = db
            .transaction(&["blobs"], TransactionMode::ReadWrite)
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;
        let blobs = transaction
            .store("blobs")
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;

        let id_js = serde_wasm_bindgen
            ::to_value(&id)
            .map_err(|e| IndexedDbBlobStorageError::SerializationError(e.to_string()))?;

        let data_array = js_sys::Uint8Array::from(data);

        blobs
            .put(&data_array, Some(&id_js)).await
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;

        transaction
            .done().await
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;

        Ok(())
    }

    async fn retrieve_blob(&self, id: &BlobId) -> Result<Vec<u8>, Self::Error> {
        let db = get_db().await.map_err(|e|
            IndexedDbBlobStorageError::IndexedDbError(format!("{:?}", e))
        )?;
        let transaction = db
            .transaction(&["blobs"], TransactionMode::ReadOnly)
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;
        let blobs = transaction
            .store("blobs")
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;

        let id_js = serde_wasm_bindgen
            ::to_value(id)
            .map_err(|e| IndexedDbBlobStorageError::SerializationError(e.to_string()))?;

        let data_js = blobs
            .get(id_js).await
            .map_err(|e| IndexedDbBlobStorageError::IndexedDbError(e.to_string()))?;

        let data_js = data_js.ok_or(IndexedDbBlobStorageError::NotFound)?;

        let array = js_sys::Uint8Array::new(&data_js);
        let vec = array.to_vec();

        Ok(vec)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IndexedDbBlobStorageError {
    #[error("IndexedDb error: {0}")] IndexedDbError(String),
    #[error("Serialization error: {0}")] SerializationError(String),
    #[error("Blob not found")] NotFound,
}
