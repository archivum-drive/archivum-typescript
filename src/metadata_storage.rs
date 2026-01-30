use archivum_core::state::sync::{
    LocalMetadataStore,
    RepoState,
    event::{ EventId, RepoEventContainer },
};
use crate::db::get_db;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

pub struct IndexedDbMetadataStorage;

impl LocalMetadataStore for IndexedDbMetadataStorage {
    type Error = IndexedDbMetadataStorageError;

    async fn save_event(&mut self, event: RepoEventContainer) -> Result<(), Self::Error> {
        let db = get_db().await.map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(format!("{:?}", e)))?;
        let transaction = db.transaction(&["events"], TransactionMode::ReadWrite)
            .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        let events = transaction.store("events")
            .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        
        // Fix: Use serde_json to ensure the key is a string. struct-based keys become JS Objects which are invalid in IndexedDB.
        let id_str = serde_json::to_string(&event.get_id())
            .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
        let id_js = JsValue::from_str(&id_str);

        let event_js = serde_wasm_bindgen::to_value(&event)
            .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
            
        events.put(&event_js, Some(&id_js)).await
            .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
            
        transaction.done().await
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
             
        Ok(())
    }

    async fn load_event(&self, id: &EventId) -> Result<RepoEventContainer, Self::Error> {
        let db = get_db().await.map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(format!("{:?}", e)))?;
        let transaction = db.transaction(&["events"], TransactionMode::ReadOnly)
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        let events = transaction.store("events")
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;

        // Fix: Match key generation from save_event
        let id_str = serde_json::to_string(id)
            .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
        let id_js = JsValue::from_str(&id_str);

        let event_js = events.get(id_js).await
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
             
        let event_js = event_js.ok_or_else(|| 
             IndexedDbMetadataStorageError::IndexedDbError(format!("Event {:?} not found", id))
        )?;

        let event: RepoEventContainer = serde_wasm_bindgen::from_value(event_js)
            .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
            
        Ok(event)
    }

    async fn save_sync_state(&mut self, state: RepoState) -> Result<(), Self::Error> {
        let db = get_db().await.map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(format!("{:?}", e)))?;
        let transaction = db.transaction(&["meta"], TransactionMode::ReadWrite)
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        let meta = transaction.store("meta")
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
             
        let state_js = serde_wasm_bindgen::to_value(&state)
            .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
        let key_js = JsValue::from_str("sync_state");
        
        meta.put(&state_js, Some(&key_js)).await
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;

        transaction.done().await
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        Ok(())
    }

    async fn load_sync_state(&self) -> Result<RepoState, Self::Error> {
        let db = get_db().await.map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(format!("{:?}", e)))?;
        let transaction = db.transaction(&["meta"], TransactionMode::ReadOnly)
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
        let meta = transaction.store("meta")
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
             
        let key_js = JsValue::from_str("sync_state");
        let state_js = meta.get(key_js).await
             .map_err(|e| IndexedDbMetadataStorageError::IndexedDbError(e.to_string()))?;
             
        match state_js {
            None => Ok(RepoState::default()),
            Some(js) => {
                let state: RepoState = serde_wasm_bindgen::from_value(js)
                    .map_err(|e| IndexedDbMetadataStorageError::SerializationError(e.to_string()))?;
                Ok(state)
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IndexedDbMetadataStorageError {
    #[error("IndexedDb error: {0}")] IndexedDbError(String),
    #[error("Serialization error: {0}")] SerializationError(String),
}
