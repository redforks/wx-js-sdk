use gloo_console::{error, log};
use gloo_net::http::Request;
use serde::Deserialize;
use serde_wasm_bindgen::to_value as to_js_value;
use snafu::{prelude::*, OptionExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use wx_js_sdk::{check_js_api, Config};

type Result<T> = std::result::Result<T, snafu::Whatever>;

fn current_url_without_hash() -> Result<String> {
    let w = window().whatever_context("get window global object")?;
    let location = w.location();
    let url = match location.href() {
        Err(e) => whatever!(
            "get current url failed: {}",
            js_sys::Error::from(e).message()
        ),
        Ok(v) => v,
    };
    let url = url
        .split('#')
        .next()
        .whatever_context("get url without hash")?;
    Ok(url.to_owned())
}

fn main() {
    spawn_local(async { handle_err(go().await) });
}

fn handle_err(rv: Result<()>) {
    match rv {
        Ok(_) => {}
        Err(e) => {
            error!(format!("{:?}", e));
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
    log!(format!("{:?}", &resp));
    config_jsapi(resp).await?;
    log!("config jsapi succeed");
    do_check_js_api().await?;
    log!("check jsapi succeed");

    Ok(())
}

async fn config_jsapi(sign: SignUrlResponse) -> Result<()> {
    let config = Config {
        debug: true,
        app_id: "wx1234567890".to_owned(),
        timestamp: sign.timestamp,
        nonce_str: sign.noncestr,
        signature: sign.sign,
        js_api_list: vec![
            "uploadImage".to_owned(),
            "chooseImage".to_owned(),
            "downloadImage".to_owned(),
        ],
    };

    if let Err(err) =
        wx_js_sdk::config(to_js_value(&config).whatever_context("config object to js")?).await
    {
        whatever!("config failed: {:?}", err);
    }
    Ok(())
}

async fn do_check_js_api() -> Result<()> {
    match check_js_api(vec!["chooseImage".to_owned()]).await {
        Err(err) => {
            whatever!("check js-api: {:?}", err);
        }
        _ => (),
    }
    Ok(())
}

async fn sign_url() -> Result<SignUrlResponse> {
    let url = current_url_without_hash()?;
    log!("Sign url", &url);
    let req = whatever!(
        Request::post("/api/wx/jsapi/sign-url").body(url),
        "create request"
    );
    let resp = whatever!(req.send().await, "send request");
    let resp = whatever!(resp.json::<SignUrlResponse>().await, "decode response");
    Ok(resp)
}
