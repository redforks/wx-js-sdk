use serde::{de::DeserializeOwned, Deserialize, Serialize};
use snafu::prelude::*;
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

trait WxResponse: Sized {
    fn err_msg(&self) -> &str;

    fn take_err_msg(self) -> String {
        self.err_msg().to_owned()
    }

    fn to_result(self) -> Result<Self, JSApiError> {
        if self.err_msg().ends_with(":ok") {
            Ok(self)
        } else {
            Err(JSApiError::ApiError {
                message: self.take_err_msg(),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct ChooseImageResult {
    #[serde(rename = "localIds")]
    pub local_ids: Vec<String>,
    #[serde(rename = "errMsg")]
    err_msg: String,
}

impl WxResponse for ChooseImageResult {
    fn err_msg(&self) -> &str {
        self.err_msg.as_ref()
    }

    fn take_err_msg(self) -> String {
        self.err_msg
    }
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
    #[serde(rename = "errMsg")]
    err_msg: String,
}

impl WxResponse for UploadImageResult {
    fn err_msg(&self) -> &str {
        self.err_msg.as_ref()
    }

    fn take_err_msg(self) -> String {
        self.err_msg
    }
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

async fn call<Req, Resp>(options: &Req) -> Result<Resp, JSApiError>
where
    Req: Serialize,
    Resp: DeserializeOwned + WxResponse,
{
    use serde_wasm_bindgen::{from_value as from_js_value, to_value as to_js_value};

    let options = whatever!(to_js_value(&options), "options to js");
    let rv = inner::choose_image(options).await;
    let rv = whatever!(from_js_value::<Resp>(rv), "convert response from js");
    rv.to_result()
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
    call(options).await
}

pub async fn upload_image(options: &UploadImageOptios) -> Result<UploadImageResult, JSApiError> {
    call(options).await
}
