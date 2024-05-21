use serde::Serialize;
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = wx_api, catch)]
    pub async fn config(config: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = wx_api, js_name=checkJsApi, catch)]
    pub async fn check_js_api(api_list: Vec<String>) -> Result<JsValue, JsValue>;
}
