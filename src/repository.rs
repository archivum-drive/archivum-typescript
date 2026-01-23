use wasm_bindgen::{ prelude::wasm_bindgen, JsValue };

use archivum_core::{
    blob::{ BlobId, BlobRecord },
    node::{ Node, NodeId },
    node_type::NodeType,
    state::{
        ClientId,
        Repository as CoreRepository,
        node_status::NodeStatus,
        sync::RemoteMetadataStore,
    },
    tag::{ TagColors, TagId, TagRecord },
};

use crate::{ metadata_storage::{ LocalstorageMetadataStorage } };

#[wasm_bindgen]
pub struct Repository {
    inner: CoreRepository<LocalstorageMetadataStorage>,
    metadata_storage: RemoteMetadataStore,
}

#[wasm_bindgen]
impl Repository {
    //
    // Constructor and load/save
    //
    #[wasm_bindgen(constructor)]
    pub fn new(
        client_id: String,
        metadata_server_url: String,
        blob_store_url: String
    ) -> Repository {
        // Better panic messages in browser console
        console_error_panic_hook::set_once();

        Repository {
            inner: CoreRepository::new(
                ClientId::parse_str(&client_id).unwrap(),
                LocalstorageMetadataStorage,
                blob_store_url
            ),
            metadata_storage: RemoteMetadataStore::new(metadata_server_url),
        }
    }

    #[wasm_bindgen(js_name = "pullRemote")]
    pub async fn pull_remote(&mut self) -> Result<(), JsValue> {
        self.inner
            .pull_remote(&self.metadata_storage).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = "pushRemote")]
    pub async fn push_remote(&mut self) -> Result<(), JsValue> {
        self.inner
            .push_remote(&self.metadata_storage).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = "loadLocal")]
    pub async fn load_local(&mut self) -> Result<(), JsValue> {
        self.inner.load_local().await.map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    //
    // Upsert Tags and Nodes
    //

    /// Returns the `NodeId` of the upserted node.
    #[wasm_bindgen(js_name = "createNode")]
    pub async fn create_node(&mut self, title: String, node_data: NodeType) -> Result<(), JsValue> {
        self.inner
            .create_node(title, node_data).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "createTag")]
    pub async fn create_tag(
        &mut self,
        path: Vec<String>,
        color: Option<TagColors>
    ) -> Result<(), JsValue> {
        self.inner.create_tag(path, color).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    //
    // getters
    //

    #[wasm_bindgen(js_name = "getAllNodes")]
    pub fn get_all_nodes(&self) -> Result<Vec<Node>, JsValue> {
        Ok(self.inner.iter_nodes().collect::<Vec<Node>>())
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

    #[wasm_bindgen(js_name = "getBlobRecord")]
    pub fn get_blob_record(&self, blob_id: BlobId) -> Option<BlobRecord> {
        self.inner.get_blob_record(&blob_id).cloned()
    }

    #[wasm_bindgen(js_name = "getNodeStatus")]
    pub fn get_node_status(&self, node_id: NodeId) -> Option<NodeStatus> {
        self.inner.get_node_status(&node_id)
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
    // update operations
    //
    #[wasm_bindgen(js_name = "renameNode")]
    pub async fn rename_node(&mut self, node_id: NodeId, new_title: String) -> Result<(), JsValue> {
        self.inner
            .rename_node(node_id, new_title).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
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
    pub fn get_nodes_with_tag(&self, tag_id: TagId) -> Vec<Node> {
        let Some(node_ids) = self.inner.get_nodes_with_tag(tag_id) else {
            return Vec::new();
        };

        let mut nodes = Vec::new();

        node_ids.iter().for_each(|node_id| {
            if let Some(node) = self.inner.get_node(*node_id) {
                nodes.push(node);
            }
        });

        nodes
    }

    //
    // Data Blob operations
    //

    #[wasm_bindgen(js_name = "uploadBlob")]
    pub async fn upload_data_as_blob(&mut self, data: &[u8]) -> Result<BlobId, JsValue> {
        let blob_id = self.inner
            .upload_data_as_blob(data).await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(blob_id)
    }

    #[wasm_bindgen(js_name = "downloadBlob")]
    pub async fn get_blob_data(&mut self, blob_id: BlobId) -> Result<Vec<u8>, JsValue> {
        self.inner.get_blob_data(&blob_id).await.map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }
}
