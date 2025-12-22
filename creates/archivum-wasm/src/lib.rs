use wasm_bindgen::prelude::*;

use archivum_core::{
    node::{ NodeId, NodeRecord },
    node_type::NodeType,
    repository::Repository as CoreRepository,
    tag::{ TagId, TagRecord },
};

fn tag_to_js(tag: &TagRecord) -> Result<JsValue, JsValue> {
    let obj = js_sys::Object::new();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("id"),
        &JsValue::from_f64(tag.get_id().0 as f64)
    )?;

    let path_arr = js_sys::Array::new();
    for p in tag.get_path().iter() {
        path_arr.push(&JsValue::from_str(p));
    }
    js_sys::Reflect::set(&obj, &JsValue::from_str("path"), &path_arr)?;

    Ok(obj.into())
}

fn node_to_js(node: &NodeRecord) -> Result<JsValue, JsValue> {
    let obj = js_sys::Object::new();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("id"),
        &JsValue::from_f64(node.get_id().0 as f64)
    )?;

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("date_created"),
        &JsValue::from_str(&node.get_date_created())
    )?;

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("date_updated"),
        &JsValue::from_str(&node.get_date_updated())
    )?;

    let data_js = serde_wasm_bindgen
        ::to_value(node.get_data())
        .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
    js_sys::Reflect::set(&obj, &JsValue::from_str("data"), &data_js)?;

    let tag_ids_arr = js_sys::Array::new();
    for tag_id in node.get_tags().iter() {
        tag_ids_arr.push(&JsValue::from_f64(tag_id.0 as f64));
    }
    js_sys::Reflect::set(&obj, &JsValue::from_str("tag_ids"), &tag_ids_arr)?;

    Ok(obj.into())
}

// --- Repository
#[wasm_bindgen]
pub struct Repository {
    inner: CoreRepository,
}

#[wasm_bindgen]
impl Repository {
    //
    // Constructor and load/save
    //
    #[wasm_bindgen(constructor)]
    pub fn new() -> Repository {
        // Better panic messages in browser console
        console_error_panic_hook::set_once();

        Repository {
            inner: CoreRepository::new(),
        }
    }

    #[wasm_bindgen(js_name = "loadFromJson")]
    pub fn load_from_json(json: String) -> Result<Repository, JsValue> {
        let inner = CoreRepository::load_from_json(&json).map_err(|e|
            JsValue::from_str(&format!("{e:?}"))
        )?;

        Ok(Repository { inner })
    }

    #[wasm_bindgen(js_name = "saveToJson")]
    pub fn save_to_json(&self) -> Result<String, JsValue> {
        self.inner.save_to_json().map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    //
    // Upsert Tags and Nodes
    //
    #[wasm_bindgen(js_name = "upsertNode")]
    pub fn upsert_node(
        &mut self,
        node_id: u32,
        node_data: JsValue,
        date_created: String,
        date_updated: String
    ) -> Result<(), JsValue> {
        let id = NodeId(node_id);

        let node_data: NodeType = serde_wasm_bindgen
            ::from_value(node_data)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        let existing_tag_ids = self.inner
            .get_node(id)
            .map(|n| n.get_tags().clone())
            .unwrap_or_default();

        let node = NodeRecord::new(id, node_data, existing_tag_ids, date_created, date_updated);

        self.inner.upsert_node(node).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "upsertTag")]
    pub fn upsert_tag(&mut self, tag_id: u32, path: Vec<String>) -> Result<(), JsValue> {
        let id = TagId(tag_id);

        let tag = TagRecord::new(id, path, None);

        self.inner.upsert_tag(tag).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteTag")]
    pub fn delete_tag(&mut self, tag_id: u32) -> Result<(), JsValue> {
        let id = TagId(tag_id);
        self.inner.delete_tag(id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "getTag")]
    pub fn get_tag(&self, tag_id: u32) -> Result<Option<JsValue>, JsValue> {
        let id = TagId(tag_id);
        match self.inner.get_tag(id) {
            Some(tag) => Ok(Some(tag_to_js(&tag)?)),
            None => Ok(None),
        }
    }

    #[wasm_bindgen(js_name = "getAllTags")]
    pub fn get_all_tags(&self) -> Result<js_sys::Array, JsValue> {
        let arr = js_sys::Array::new();
        for tag in self.inner.iter_tags() {
            arr.push(&tag_to_js(tag)?);
        }
        Ok(arr)
    }

    //
    // Get Nodes/Tags
    //
    #[wasm_bindgen(js_name = "getAllNodes")]
    pub fn get_all_nodes(&self) -> Result<js_sys::Array, JsValue> {
        let arr = js_sys::Array::new();
        for node in self.inner.iter_nodes() {
            arr.push(&node_to_js(node)?);
        }
        Ok(arr)
    }

    #[wasm_bindgen(js_name = "getNode")]
    pub fn get_node(&self, node_id: u32) -> Result<Option<JsValue>, JsValue> {
        let id = NodeId(node_id);
        Ok(
            self.inner
                .get_node(id)
                .map(|n| node_to_js(&n))
                .transpose()?
        )
    }

    #[wasm_bindgen(js_name = "deleteNode")]
    pub fn delete_node(&mut self, node_id: u32) -> Result<(), JsValue> {
        let id = NodeId(node_id);
        self.inner.delete_node(id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(())
    }

    //
    // Tag operations
    //
    #[wasm_bindgen(js_name = "getTagByPath")]
    pub fn get_tag_by_path(&mut self, path: Vec<String>) -> Result<u32, JsValue> {
        match self.inner.get_tag_by_path(path) {
            Ok(tag) => Ok(tag.get_id().0),
            Err(e) => Err(JsValue::from_str(&format!("{e:?}"))),
        }
    }

    #[wasm_bindgen(js_name = "tagNode")]
    pub fn tag_node(&mut self, node_id: u32, tag_id: u32) -> Result<(), JsValue> {
        let node_id = NodeId(node_id);
        let tag_id = TagId(tag_id);

        self.inner.tag_node(node_id, tag_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "untagNode")]
    pub fn untag_node(&mut self, node_id: u32, tag_id: u32) -> Result<(), JsValue> {
        let node_id = NodeId(node_id);
        let tag_id = TagId(tag_id);

        self.inner.untag_node(node_id, tag_id).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "getNodesWithTag")]
    pub fn get_nodes_with_tag(&self, tag_id: u32) -> Result<js_sys::Array, JsValue> {
        let tag_id = TagId(tag_id);
        let arr = js_sys::Array::new();

        for node in self.inner.iter_nodes() {
            if
                node
                    .get_tags()
                    .iter()
                    .any(|t| *t == tag_id)
            {
                arr.push(&node_to_js(node)?);
            }
        }

        Ok(arr)
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}
