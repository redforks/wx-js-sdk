use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use snafu::{prelude::*, OptionExt, ResultExt};
use wasm_bindgen::JsValue;

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
struct WxResponse<T> {
    // https://github.com/serde-rs/serde/issues/1879
    // can not set flatten and default the same time,
    #[serde(rename = "errMsg")]
    err_msg: String,
    /// should be Some on success
    #[serde(flatten)]
    value: Option<T>,
}

impl<T: DeserializeOwned> WxResponse<T> {
    pub fn into_result(self) -> std::result::Result<T, JSApiError> {
        if self.err_msg.ends_with(":ok") {
            Ok(self.value.whatever_context("Should have value on Ok")?)
        } else {
            Err(JSApiError::ApiError {
                message: self.err_msg,
            })
        }
    }

    pub fn js_into_result(val: JsValue) -> Result<T, JSApiError> {
        let v = Self::from_js(val)?;
        v.into_result()
    }

    fn from_js(val: JsValue) -> Result<Self, JSApiError> {
        from_value(val).whatever_context("decode response from js")
    }
}

#[derive(Deserialize)]
pub struct ChooseImageResult {
    #[serde(rename = "localIds")]
    pub local_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct UploadImageOptions {
    #[serde(rename = "localId")]
    pub local_id: String,
}

#[derive(Deserialize, Debug)]
pub struct UploadImageResult {
    #[serde(rename = "serverId")]
    pub server_id: String,
}

mod inner {
    use wasm_bindgen::{prelude::*, JsValue};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = wx_api, catch)]
        pub async fn config(config: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(js_namespace = wx_api, js_name=checkJsApi)]
        async fn check_js_api(api_list: Vec<String>) -> JsValue;

        #[wasm_bindgen(js_namespace = wx_api, js_name=chooseImage)]
        pub async fn choose_image(options: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = wx_api, js_name=uploadImage)]
        pub async fn upload_image(options: JsValue) -> JsValue;
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum JSApiError {
    /// Error on config()
    ConfigError { err: JsValue },

    /// Error returned by wx-jsapi
    #[snafu(display("{message}"))]
    ApiError { message: String },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

pub async fn config(options: &Config) -> Result<(), JSApiError> {
    use serde_wasm_bindgen::to_value as to_js_value;
    let options = whatever!(to_js_value(&options), "options to js");
    match inner::config(options).await {
        Ok(_) => Ok(()),
        Err(err) => Err(JSApiError::ConfigError { err }),
    }
}

pub async fn choose_image(options: &ChooseImageOptions) -> Result<ChooseImageResult, JSApiError> {
    {
        async move {
            let options = whatever!(to_value(&options), "options to js");
            let rv = inner::choose_image(options).await;
            WxResponse::<ChooseImageResult>::js_into_result(rv)
        }
    }
    .await
}

pub async fn upload_image(options: &UploadImageOptions) -> Result<UploadImageResult, JSApiError> {
    {
        async move {
            let options = whatever!(to_value(&options), "options to js");
            let rv = inner::upload_image(options).await;
            WxResponse::<UploadImageResult>::js_into_result(rv)
        }
    }
    .await
}
