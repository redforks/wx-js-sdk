use js_sys::wasm_bindgen::JsValue;
use snafu::{prelude::*, OptionExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, History, HtmlElement, Window};
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
    log("change url by pushState()");
    let h = history()?;
    handle_js_error(h.push_state_with_url(&JsValue::NULL, "", Some("sub/url")))?;
    log(&format!("current_url: {}", current_url()?));

    log("check js api...");
    log(&format!(
        "{:#?}",
        whatever!(
            wx_js_sdk::check_js_api(vec!["chooseImage".to_string(), "uploadImage".to_string()])
                .await,
            "check js api"
        )
    ));
    log("check js api succeed");

    log("back to previous url...");
    handle_js_error(h.go_with_delta(-1))?;
    log(&format!("current_url: {}", current_url()?));

    log("choose image...");
    if let Some(local_id) = choose_image().await? {
        log("choose image succeed");

        log("Upload image...");
        let _server_id = upload_image(local_id).await?;
        log("upload image succeed");
    } else {
        log("user cancelle");
    }

    Ok(())
}

fn window() -> Result<Window> {
    use web_sys::window;

    let w = window().whatever_context("get window global object")?;
    Ok(w)
}

fn history() -> Result<History> {
    let w = window()?;
    let h = handle_js_error(w.history())?;
    Ok(h)
}

fn current_url() -> Result<String> {
    let w = window()?;
    let location = w.location();
    Ok(whatever!(
        handle_js_error(location.href()),
        "get current page url"
    ))
}

fn document() -> Result<Document> {
    window()?.document().whatever_context("get document")
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
