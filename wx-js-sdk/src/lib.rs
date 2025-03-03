use gloo_net::http::Request;
use js_sys::JSON;
use linear_map::LinearMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use snafu::{OptionExt, ResultExt, prelude::*};
use std::sync::atomic::AtomicU8;
use wasm_bindgen::JsValue;
use web_sys::window;

type Result<T> = std::result::Result<T, JSApiError>;

#[derive(Serialize)]
struct Config {
    pub debug: bool,
    #[serde(rename = "appId")]
    pub app_id: String,
    pub timestamp: u32,
    #[serde(rename = "nonceStr")]
    pub nonce_str: String,
    pub signature: String,
    #[serde(rename = "jsApiList")]
    pub js_api_list: Vec<&'static str>,
    #[serde(rename = "openTagList")]
    pub open_tag_list: Vec<&'static str>,
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
    #[serde(rename = "errMsg", alias = "err_msg")]
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

    /// Like into_result, but map :cancel error to `Ok(None)`.
    pub fn into_cancel_result(self) -> Result<Option<T>> {
        if self.err_msg.ends_with(":ok") {
            Ok(Some(
                self.value.whatever_context("Should have value on Ok")?,
            ))
        } else if self.err_msg.ends_with(":cancel") {
            Ok(None)
        } else {
            Err(JSApiError::ApiError {
                message: self.err_msg,
            })
        }
    }

    pub fn js_into_result(val: JsValue) -> Result<T> {
        let v = Self::from_js(val)?;
        v.into_result()
    }

    pub fn js_into_cancel_result(val: JsValue) -> Result<Option<T>> {
        let v = Self::from_js(val)?;
        v.into_cancel_result()
    }

    fn from_js(val: JsValue) -> Result<Self> {
        from_value(val).whatever_context("decode response from js")
    }
}

#[derive(Deserialize, Debug)]
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
    use wasm_bindgen::{JsValue, prelude::*};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = wx_api, catch)]
        pub async fn config(config: JsValue) -> Result<(), JsValue>;

        #[wasm_bindgen(js_namespace = wx_api, js_name=checkJsApi)]
        pub async fn check_js_api(options: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = wx_api, js_name=chooseImage)]
        pub async fn choose_image(options: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = wx_api, js_name=uploadImage)]
        pub async fn upload_image(options: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = wx_api, js_name=pay)]
        pub async fn pay(options: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = wx, js_name=closeWindow)]
        pub fn close_window();
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum JSApiError {
    /// Error on config()
    #[snafu(display("初始化微信jsapi失败: {err}"))]
    ConfigError { err: String },

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

async fn call_config(options: &Config) -> Result<()> {
    use serde_wasm_bindgen::to_value as to_js_value;
    let options = whatever!(to_js_value(&options), "options to js");
    match inner::config(options).await {
        Ok(_) => Ok(()),
        Err(err) => Err(JSApiError::ConfigError {
            err: JSON::stringify(&err).map_or_else(|_| "2c35df0".to_string(), |v| v.into()),
        }),
    }
}

/// Choose image, returns local_id, return `Ok(None)` if user cancel the operation.
pub async fn choose_image(options: &ChooseImageOptions) -> Result<Option<ChooseImageResult>> {
    auto_config().await?;

    let options = whatever!(to_value(&options), "options to js");
    let rv = inner::choose_image(options).await;
    WxResponse::<ChooseImageResult>::js_into_cancel_result(rv)
}

pub async fn upload_image(options: &UploadImageOptions) -> Result<UploadImageResult> {
    auto_config().await?;

    let options = whatever!(to_value(&options), "options to js");
    let rv = inner::upload_image(options).await;
    WxResponse::<UploadImageResult>::js_into_result(rv)
}

pub use inner::close_window;

/// 客户端发起微信支付api 调用的参数
///
/// <https://pay.weixin.qq.com/wiki/doc/apiv3/apis/chapter3_1_4.shtml>
// #[derive(Serialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct PayOptions {
//     /// 应用ID
//     pub app_id: String,
//     /// 时间戳
//     pub time_stamp: String,
//     /// 随机字符串
//     pub nonce_str: String,
//     /// 订单详情扩展字符串
//     pub package: String,
//     /// 签名方式
//     pub sign_type: &'static str,
//     /// 签名
//     pub pay_sign: String,
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayRequest {
    pub app_id: String,
    pub time_stamp: String,
    pub nonce_str: String,
    pub package: String,
    pub sign_type: String,
    pub pay_sign: String,
}

pub async fn pay(options: &PayRequest) -> Result<()> {
    // auto_config().await?;

    let options = whatever!(to_value(&options), "options to js");
    let rv = inner::pay(options).await;
    WxResponse::<()>::js_into_result(rv)
}

#[derive(Serialize, Debug)]
struct CheckJsApiOptions {
    #[serde(rename = "jsApiList")]
    pub js_api_list: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct CheckJsApiResult {
    #[serde(rename = "checkResult")]
    pub check_result: LinearMap<String, bool>,
}

pub async fn check_js_api(api_list: Vec<String>) -> Result<LinearMap<String, bool>> {
    auto_config().await?;

    let options = CheckJsApiOptions {
        js_api_list: api_list,
    };
    let options = whatever!(to_value(&options), "options to js");
    let rv = inner::check_js_api(options).await;
    WxResponse::<CheckJsApiResult>::js_into_result(rv).map(|v| v.check_result)
}

const INIT_STATE_UNINITIALIZED: u8 = 0;
const INIT_STATE_INITIALIZED: u8 = 1;
const INIT_STATE_INITIALIZING: u8 = 2;

static INIT_STATE: AtomicU8 = AtomicU8::new(0);

pub async fn config(force: bool) -> Result<()> {
    if force {
        do_config().await
    } else {
        auto_config().await
    }
}

async fn do_config() -> Result<()> {
    let url = current_url_without_hash()?;
    let sign = sign_url(url).await?;
    call_config(&Config {
        debug: false,
        app_id: env!("WECHAT_APP_ID").to_owned(),
        timestamp: sign.timestamp,
        nonce_str: sign.noncestr,
        signature: sign.sign,
        js_api_list: vec!["uploadImage", "chooseImage", "downloadImage"],
        open_tag_list: vec!["wx-open-launch-weapp"],
    })
    .await
}

async fn auto_config() -> Result<()> {
    match INIT_STATE.compare_exchange(
        INIT_STATE_UNINITIALIZED,
        INIT_STATE_INITIALIZING,
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
    ) {
        Ok(_) | Err(INIT_STATE_INITIALIZING) => {
            do_config().await?;
            INIT_STATE.store(INIT_STATE_INITIALIZED, std::sync::atomic::Ordering::Relaxed);
        }
        Err(INIT_STATE_INITIALIZED) => {}
        _ => {
            whatever!("initalizing")
        }
    }
    Ok(())
}

fn handle_js_error<T>(v: std::result::Result<T, JsValue>) -> Result<T> {
    match v {
        Ok(v) => Ok(v),
        Err(e) => {
            whatever!("{}", js_sys::Error::from(e).message())
        }
    }
}

fn current_url_without_hash() -> Result<String> {
    let w = window().whatever_context("get window global object")?;
    let location = w.location();
    let url = whatever!(handle_js_error(location.href()), "get current page url");
    let url = url
        .split('#')
        .next()
        .whatever_context("get url without hash")?;
    Ok(url.to_owned())
}

#[derive(Deserialize, Debug)]
struct SignUrlResponse {
    sign: String,
    timestamp: u32,
    noncestr: String,
}

async fn sign_url(url: String) -> Result<SignUrlResponse> {
    let req = whatever!(
        Request::post("/api/wx/jsapi/sign-url").body(url),
        "create request"
    );
    let resp = whatever!(req.send().await, "send request");
    let resp = whatever!(resp.json::<SignUrlResponse>().await, "decode response");
    Ok(resp)
}
