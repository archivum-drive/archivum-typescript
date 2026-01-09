use wasm_bindgen::{ prelude::wasm_bindgen, JsValue };

use archivum_core::{
    blob_storage::{ blob::DataBlob, blob_store::ArchivumBlobServerStore },
    node::{ NodeId, NodeRecord },
    state::repository::Repository as CoreRepository,
    tag::{ TagId, TagRecord },
};

use crate::{ metadata_storage::{ LocalstorageMetadataStorage } };

#[wasm_bindgen]
pub struct Repository {
    inner: CoreRepository<LocalstorageMetadataStorage>,
    blob_store: ArchivumBlobServerStore,
}

#[wasm_bindgen]
impl Repository {
    //
    // Constructor and load/save
    //
    #[wasm_bindgen(constructor)]
    pub fn new(store_url: String) -> Repository {
        // Better panic messages in browser console
        console_error_panic_hook::set_once();

        Repository {
            inner: CoreRepository::new(LocalstorageMetadataStorage),
            blob_store: ArchivumBlobServerStore::new(store_url),
        }
    }

    #[wasm_bindgen(js_name = "loadLocal")]
    pub async fn load_local(&mut self) -> Result<(), JsValue> {
        self.inner.load_local().await.map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = "saveLocal")]
    pub async fn save_local(&mut self) -> Result<(), JsValue> {
        self.inner.save_local().await.map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    //
    // Upsert Tags and Nodes
    //

    /// Returns the `NodeId` of the upserted node.
    #[wasm_bindgen(js_name = "upsertNode")]
    pub async fn upsert_node(&mut self, node: NodeRecord) -> Result<(), JsValue> {
        self.inner.upsert_node(node).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "upsertTag")]
    pub async fn upsert_tag(&mut self, tag: TagRecord) -> Result<(), JsValue> {
        self.inner.upsert_tag(tag).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    //
    // getters
    //

    #[wasm_bindgen(js_name = "getAllNodes")]
    pub fn get_all_nodes(&self) -> Result<Vec<NodeRecord>, JsValue> {
        Ok(self.inner.iter_nodes().cloned().collect::<Vec<NodeRecord>>())
    }

    #[wasm_bindgen(js_name = "getAllTags")]
    pub fn get_all_tags(&self) -> Result<Vec<TagRecord>, JsValue> {
        Ok(self.inner.iter_tags().cloned().collect::<Vec<TagRecord>>())
    }

    #[wasm_bindgen(js_name = "getTagByPath")]
    pub fn get_tag_by_path(&mut self, path: Vec<String>) -> Result<Option<TagRecord>, JsValue> {
        let node_id = self.inner
            .get_tag_by_path(path)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        let tag = self.inner.get_tag(node_id);

        Ok(tag)
    }

    //
    // delete operations
    //
    #[wasm_bindgen(js_name = "deleteNode")]
    pub async fn delete_node(&mut self, node_id: NodeId) -> Result<(), JsValue> {
        self.inner.delete_node(node_id).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteTag")]
    pub async fn delete_tag(&mut self, tag_id: TagId) -> Result<(), JsValue> {
        self.inner.delete_tag(tag_id).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    //
    // tagging operations
    //

    #[wasm_bindgen(js_name = "tagNode")]
    pub async fn tag_node(&mut self, node_id: NodeId, tag_id: TagId) -> Result<(), JsValue> {
        self.inner
            .tag_node(node_id, tag_id).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "untagNode")]
    pub async fn untag_node(&mut self, node_id: NodeId, tag_id: TagId) -> Result<(), JsValue> {
        self.inner
            .untag_node(node_id, tag_id).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    //
    // querying
    //

    #[wasm_bindgen(js_name = "getChildTags")]
    pub fn get_child_tags(&self, parent: TagId) -> Option<Vec<TagRecord>> {
        let tag_ids = self.inner.get_child_tags(parent);

        tag_ids.map(|ids| {
            ids.iter()
                .filter_map(|id| self.inner.get_tag(*id))
                .collect()
        })
    }

    #[wasm_bindgen(js_name = "getNodesWithTag")]
    pub fn get_nodes_with_tag(&self, tag_id: TagId) -> Vec<NodeRecord> {
        let Some(node_ids) = self.inner.get_nodes_with_tag(tag_id) else {
            return Vec::new();
        };

        let mut nodes = Vec::new();

        node_ids.iter().for_each(|node_id| {
            if let Some(node) = self.inner.get_node(*node_id).cloned() {
                nodes.push(node);
            }
        });

        nodes
    }

    //
    // Get next IDs
    //

    #[wasm_bindgen(js_name = "getNextNodeId")]
    pub fn get_next_node_id(&mut self) -> NodeId {
        self.inner.get_next_node_id()
    }
    #[wasm_bindgen(js_name = "getNextTagId")]
    pub fn get_next_tag_id(&mut self) -> TagId {
        self.inner.get_next_tag_id()
    }

    //
    // Data Blob operations
    //

    #[wasm_bindgen(js_name = "uploadBlob")]
    pub async fn upload_blob(&mut self, data: &[u8]) -> Result<DataBlob, JsValue> {
        self.inner
            .upload_data(&mut self.blob_store, data).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = "downloadBlob")]
    pub async fn download_blob(&mut self, blob: DataBlob) -> Result<Vec<u8>, JsValue> {
        blob.retrieve_data(&self.blob_store).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }
}
