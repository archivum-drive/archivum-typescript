use wasm_bindgen::prelude::*;

use archivum_core::{
    node::{NodeId, NodeRecord},
    repository::Repository as CoreRepository,
    tag::{TagId, TagRecord},
};

fn uuid_str_to_u128(s: &str) -> Result<u128, JsValue> {
    let u = uuid::Uuid::parse_str(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(u.as_u128())
}

fn u128_to_uuid_str(v: u128) -> String {
    uuid::Uuid::from_u128(v).to_string()
}

fn node_id_from_uuid(s: &str) -> Result<NodeId, JsValue> {
    Ok(NodeId(uuid_str_to_u128(s)?))
}

fn tag_id_from_uuid(s: &str) -> Result<TagId, JsValue> {
    Ok(TagId(uuid_str_to_u128(s)?))
}

#[wasm_bindgen]
pub struct Repository {
    inner: CoreRepository,
}

#[wasm_bindgen]
impl Repository {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Repository {
        // Better panic messages in browser console
        console_error_panic_hook::set_once();

        Repository {
            inner: CoreRepository::new(),
        }
    }

    #[wasm_bindgen(js_name = "upsertNode")]
    pub fn upsert_node(
        &mut self,
        node_id_uuid: String,
        date_created: String,
        date_updated: String,
    ) -> Result<(), JsValue> {
        let id = node_id_from_uuid(&node_id_uuid)?;

        let node = NodeRecord::new(id, smallvec::smallvec![], date_created, date_updated);

        self.inner
            .upsert_node(node)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "upsertTag")]
    pub fn upsert_tag(&mut self, tag_id_uuid: String, path: Vec<String>) -> Result<(), JsValue> {
        let id = tag_id_from_uuid(&tag_id_uuid)?;

        let tag = TagRecord::new(id, path, None);

        self.inner
            .upsert_tag(tag)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "getTagByPath")]
    pub fn get_tag_by_path(&mut self, path: Vec<String>) -> Result<Option<String>, JsValue> {
        match self.inner.get_tag_by_path(path) {
            Ok(tag) => Ok(Some(u128_to_uuid_str(tag.0))),
            Err(e) => Err(JsValue::from_str(&format!("{e:?}"))),
        }
    }

    #[wasm_bindgen(js_name = "tagNode")]
    pub fn tag_node(&mut self, node_id_uuid: String, tag_id_uuid: String) -> Result<(), JsValue> {
        let node_id = node_id_from_uuid(&node_id_uuid)?;
        let tag_id = tag_id_from_uuid(&tag_id_uuid)?;

        self.inner
            .tag_node(node_id, tag_id)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "listNodeIds")]
    pub fn list_node_ids(&self) -> Vec<String> {
        self.inner
            .iter_nodes()
            .map(|n| u128_to_uuid_str(n.get_id().0))
            .collect()
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}
