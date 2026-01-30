use rexie::Rexie;
use wasm_bindgen::JsValue;

pub async fn get_db() -> Result<Rexie, JsValue> {
    let rexie = Rexie::builder("archivum")
        .version(1)
        .add_object_store(rexie::ObjectStore::new("events"))
        .add_object_store(rexie::ObjectStore::new("meta"))
        .add_object_store(rexie::ObjectStore::new("blobs"))
        .build().await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(rexie)
}
