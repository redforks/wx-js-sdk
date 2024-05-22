use gloo_net::http::Request;
use js_sys::wasm_bindgen::JsValue;
use serde::Deserialize;
use serde_wasm_bindgen::{from_value as from_js_value, to_value as to_js_value};
use snafu::{prelude::*, OptionExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Document, HtmlElement};
use wx_js_sdk::{
    ChooseImageOptions, ChooseImageResult, Config, UploadImageOptions, UploadImageResult,
};

type Result<T> = std::result::Result<T, snafu::Whatever>;

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

fn main() {
    console_error_panic_hook::set_once();
    spawn_local(async { handle_err(go().await) });
}

fn handle_err(rv: Result<()>) {
    match rv {
        Ok(_) => {}
        Err(e) => {
            error_to_dom(&format!("Failed: {:?}", e));
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SignUrlResponse {
    sign: String,
    timestamp: u32,
    noncestr: String,
}

async fn go() -> Result<()> {
    // Get current browser url
    let resp = sign_url().await?;
    log_to_dom(&format!("{:?}", &resp));
    config_jsapi(resp).await?;

    log_to_dom("choose image...");
    let local_id = choose_image().await?;
    log_to_dom("choose image succeed");

    log_to_dom("Upload image...");
    let _server_id = upload_image(local_id).await?;
    log_to_dom("upload image succeed");

    Ok(())
}

async fn config_jsapi(sign: SignUrlResponse) -> Result<()> {
    let config = Config {
        debug: true,
        app_id: "wx823ba6aecdee1404".to_owned(),
        timestamp: sign.timestamp,
        nonce_str: sign.noncestr,
        signature: sign.sign,
        js_api_list: vec![
            "uploadImage".to_owned(),
            "chooseImage".to_owned(),
            "downloadImage".to_owned(),
        ],
    };

    whatever!(wx_js_sdk::config(&config).await, "config");
    Ok(())
}

async fn sign_url() -> Result<SignUrlResponse> {
    let url = current_url_without_hash()?;
    log_to_dom(&format!("Sign url: {}", &url));
    let req = whatever!(
        Request::post("/api/wx/jsapi/sign-url").body(url),
        "create request"
    );
    let resp = whatever!(req.send().await, "send request");
    let resp = whatever!(resp.json::<SignUrlResponse>().await, "decode response");
    Ok(resp)
}

fn document() -> Result<Document> {
    let w = window().whatever_context("get window global object")?;
    w.document().whatever_context("get document")
}

fn body(doc: &Document) -> Result<HtmlElement> {
    let r = doc.body().whatever_context("get body")?;
    Ok(r)
}

fn _log_to_dom(s: &str) -> Result<()> {
    let doc = document()?;
    let body = body(&doc)?;
    let p = whatever!(
        handle_js_error(doc.create_element("p")),
        "Create <p> element"
    );
    p.set_text_content(Some(s));
    whatever!(handle_js_error(body.append_child(&p)), "Append p node");
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

fn log_to_dom(s: &str) {
    handle_err(_log_to_dom(s))
}

fn error_to_dom(s: &str) {
    // TODO: set different css
    log_to_dom(s)
}

async fn choose_image() -> Result<String> {
    let options = ChooseImageOptions { count: 1 };
    let ids = whatever!(wx_js_sdk::choose_image(&options).await, "choose image");
    let mut ids = ids.local_ids;
    log_to_dom(&format!("{:?}", ids));
    let r = ids.remove(0);
    Ok(r)
}

async fn upload_image(local_id: String) -> Result<String> {
    let options = UploadImageOptions { local_id };
    let res = whatever!(wx_js_sdk::upload_image(&options).await, "upload image");
    log_to_dom(&format!("{:?}", &res));
    Ok(res.server_id)
}
