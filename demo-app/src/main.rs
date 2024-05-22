use js_sys::wasm_bindgen::JsValue;
use snafu::{prelude::*, OptionExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Document, HtmlElement};
use wx_js_sdk::{ChooseImageOptions, UploadImageOptions};

type Result<T> = std::result::Result<T, snafu::Whatever>;

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

async fn go() -> Result<()> {
    log("choose image...");
    if let Some(local_id) = choose_image().await? {
        log("choose image succeed");

        log("Upload image...");
        let _server_id = upload_image(local_id).await?;
        log("upload image succeed");
    } else {
        log("user cancelled");
    }

    Ok(())
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

fn log(s: &str) {
    handle_err(_log_to_dom(s))
}

fn error_to_dom(s: &str) {
    // TODO: set different css
    log(s)
}

async fn choose_image() -> Result<Option<String>> {
    let options = ChooseImageOptions { count: 1 };
    let ids = whatever!(wx_js_sdk::choose_image(&options).await, "choose image");
    log(&format!("{:?}", ids));
    Ok(ids.and_then(|ids| ids.local_ids.first().cloned()))
}

async fn upload_image(local_id: String) -> Result<String> {
    let options = UploadImageOptions { local_id };
    let res = whatever!(wx_js_sdk::upload_image(&options).await, "upload image");
    log(&format!("{:?}", &res));
    Ok(res.server_id)
}
