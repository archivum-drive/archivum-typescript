use archivum_core::state::metadata_storage::{ LocalMetadataStorage };
use gloo_storage::{ LocalStorage, Storage };

pub struct LocalstorageMetadataStorage;

impl LocalMetadataStorage for LocalstorageMetadataStorage {
    type Error = LocalstorageMetadataStorageError;

    async fn save_event(
        &mut self,
        event: archivum_core::state::sync::event::RepoEvent
    ) -> Result<(), Self::Error> {
        let mut events = self.load_events().await.unwrap_or_else(|_| vec![]);
        events.push(event);

        let serialized = serde_json
            ::to_string(&events)
            .map_err(|e|
                LocalstorageMetadataStorageError::LocalstorageError(
                    format!("Serialization error: {e:?}")
                )
            )?;
        LocalStorage::set("archivum_repo_events", serialized).map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(
                format!("Localstorage set error: {e:?}")
            )
        )?;

        Ok(())
    }

    async fn load_events(
        &self
    ) -> Result<Vec<archivum_core::state::sync::event::RepoEvent>, Self::Error> {
        let serialized = LocalStorage::get::<String>("archivum_repo_events").map_err(|e|
            LocalstorageMetadataStorageError::LocalstorageError(
                format!("Localstorage get error: {e:?}")
            )
        )?;
        let events = serde_json
            ::from_str(&serialized)
            .map_err(|e|
                LocalstorageMetadataStorageError::LocalstorageError(
                    format!("Deserialization error: {e:?}")
                )
            )?;

        Ok(events)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LocalstorageMetadataStorageError {
    #[error("Localstorage error: {0}")] LocalstorageError(String),
}
