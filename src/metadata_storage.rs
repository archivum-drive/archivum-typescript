use archivum_core::state::sync::{
    LocalMetadataStore,
    RepoState,
    event::{ EventId, RepoEventContainer },
};
use gloo_storage::{ LocalStorage, Storage, errors::StorageError };

pub struct LocalstorageMetadataStorage;

impl LocalMetadataStore for LocalstorageMetadataStorage {
    type Error = LocalstorageMetadataStorageError;

    async fn save_event(&mut self, event: RepoEventContainer) -> Result<(), Self::Error> {
        let mut events = get_all_events()?;
        events.push(event);

        LocalStorage::set("archivum_repo_events", events).map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(e.to_string())
        )?;
        Ok(())
    }

    // todo: needs rewrite for IndexedDB or similar
    async fn load_event(&self, id: &EventId) -> Result<RepoEventContainer, Self::Error> {
        let events = get_all_events()?;

        let event = events
            .into_iter()
            .find(|e| e.get_id() == id)
            .ok_or_else(||
                LocalstorageMetadataStorageError::LocalstorageError(
                    format!("Event with ID {:?} not found", id)
                )
            )?;

        Ok(event)
    }

    async fn save_sync_state(&mut self, state: RepoState) -> Result<(), Self::Error> {
        LocalStorage::set("archivum_repo_sync_state", state).map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(e.to_string())
        )?;
        Ok(())
    }

    async fn load_sync_state(&self) -> Result<RepoState, Self::Error> {
        let state = LocalStorage::get("archivum_repo_sync_state");
        match state {
            Ok(s) => Ok(s),
            Err(StorageError::KeyNotFound(_)) => Ok(RepoState::default()), // Return default state if not found
            Err(_) =>
                Err(
                    LocalstorageMetadataStorageError::LocalstorageError(
                        "Failed to load sync state".to_string()
                    )
                ),
        }
    }
}

fn get_all_events() -> Result<Vec<RepoEventContainer>, LocalstorageMetadataStorageError> {
    match LocalStorage::get("archivum_repo_events") {
        Ok(events) => Ok(events),
        Err(StorageError::KeyNotFound(_)) => Ok(Vec::new()), // No events stored yet
        Err(e) => Err(LocalstorageMetadataStorageError::LocalstorageError(e.to_string())),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LocalstorageMetadataStorageError {
    #[error("Localstorage error: {0}")] LocalstorageError(String),
}
