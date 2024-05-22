use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct Config {
    pub debug: bool,
    #[serde(rename = "appId")]
    pub app_id: String,
    pub timestamp: u32,
    #[serde(rename = "nonceStr")]
    pub nonce_str: String,
    pub signature: String,
    #[serde(rename = "jsApiList")]
    pub js_api_list: Vec<String>,
}

#[derive(Serialize)]
pub struct ChooseImageOptions {
    pub count: u8,
}

impl Default for ChooseImageOptions {
    fn default() -> Self {
        Self { count: 9 }
    }
}

#[derive(Deserialize)]
pub struct ChooseImageResult {
    #[serde(rename = "localIds")]
    pub local_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct UploadImageOptios {
    #[serde(rename = "localId")]
    pub local_id: String,
}

#[derive(Deserialize, Debug)]
pub struct UploadImageResult {
    #[serde(rename = "serverId")]
    pub server_id: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = wx_api, catch)]
    pub async fn config(config: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = wx_api, js_name=checkJsApi, catch)]
    pub async fn check_js_api(api_list: Vec<String>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = wx_api, js_name=chooseImage, catch)]
    pub async fn choose_image(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = wx_api, js_name=uploadImage, catch)]
    pub async fn upload_image(options: JsValue) -> Result<JsValue, JsValue>;
}
