use archivum_core::{ blob::DataBlob, node::{ NodeId, NodeRecord }, tag::{ TagId, TagRecord } };
use wasm_bindgen::{ prelude::wasm_bindgen, JsValue };

use archivum_core::state::repository::Repository as CoreRepository;

use crate::network_blob_store::NetworkBlobStore;

#[wasm_bindgen]
pub struct Repository {
    inner: CoreRepository,
    blob_store: NetworkBlobStore,
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
            inner: CoreRepository::new(),
            blob_store: NetworkBlobStore::new(store_url),
        }
    }

    #[wasm_bindgen(js_name = "loadFromJson")]
    pub fn load_from_json(store_url: String, json: String) -> Result<Repository, JsValue> {
        let inner = CoreRepository::load_from_json(&json).map_err(|e|
            JsValue::from_str(&format!("{e:?}"))
        )?;

        Ok(Repository { inner, blob_store: NetworkBlobStore::new(store_url) })
    }

    #[wasm_bindgen(js_name = "saveToJson")]
    pub fn save_to_json(&self) -> Result<String, JsValue> {
        self.inner.save_to_json().map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    //
    // Upsert Tags and Nodes
    //

    /// Returns the `NodeId` of the upserted node.
    #[wasm_bindgen(js_name = "upsertNode")]
    pub fn upsert_node(&mut self, node: NodeRecord) -> Result<(), JsValue> {
        self.inner.upsert_node(node).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "upsertTag")]
    pub fn upsert_tag(&mut self, tag: TagRecord) -> Result<(), JsValue> {
        self.inner.upsert_tag(tag).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

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
    pub fn delete_node(&mut self, node_id: NodeId) -> Result<(), JsValue> {
        self.inner.delete_node(node_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteTag")]
    pub fn delete_tag(&mut self, tag_id: TagId) -> Result<(), JsValue> {
        self.inner.delete_tag(tag_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    //
    // tagging operations
    //

    #[wasm_bindgen(js_name = "tagNode")]
    pub fn tag_node(&mut self, node_id: NodeId, tag_id: TagId) -> Result<(), JsValue> {
        self.inner.tag_node(node_id, tag_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "untagNode")]
    pub fn untag_node(&mut self, node_id: NodeId, tag_id: TagId) -> Result<(), JsValue> {
        self.inner.untag_node(node_id, tag_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
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
