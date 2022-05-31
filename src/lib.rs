use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

use futures_util::future::FutureExt;
use log::Level;
use magic_wormhole::{Code, transfer, transit, Wormhole};
use wasm_bindgen::prelude::*;

mod event;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct WormholeConfig {
    rendezvous_url: String,
    relay_url: String,
}

#[wasm_bindgen]
impl WormholeConfig {
    pub fn new(rendezvous_url: String, relay_url: String) -> WormholeConfig {
        wasm_logger::init(wasm_logger::Config::new(Level::Info));

        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        WormholeConfig {
            rendezvous_url,
            relay_url,
        }
    }
}

#[wasm_bindgen]
pub async fn send(config: WormholeConfig, file: web_sys::Blob, file_name: String, cancel: js_sys::Promise, progress_handler: js_sys::Function) -> Result<JsValue, JsValue> {
    let event_handler = Rc::new(Box::new(move |e: event::Event| {
        progress_handler.call1(&JsValue::null(), &JsValue::from_serde(&e).unwrap()).expect("progress_handler call should succeed");
    }) as Box<dyn Fn(event::Event)>);

    let file_content = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await?;
    let array = js_sys::Uint8Array::new(&file_content);
    let len = array.byte_length() as u64;
    let data_to_send: Vec<u8> = array.to_vec();

    let (server_welcome, connector) = Wormhole::connect_without_code(
        transfer::APP_CONFIG.rendezvous_url(config.rendezvous_url.into()),
        2,
    ).await.map_err(stringify)?;

    event_handler(event::code(server_welcome.code.0));

    let ph = event_handler.clone();
    let wormhole = connector.await.map_err(stringify)?;
    transfer::send_file(
        wormhole,
        url::Url::parse(&config.relay_url).unwrap(),
        &mut &data_to_send[..],
        PathBuf::from(file_name),
        len,
        transit::Abilities::FORCE_RELAY,
        |_info, _address| {
            event_handler(event::connected());
        },
        move |cur, total| {
            ph(event::progress(cur, total));
        },
        wasm_bindgen_futures::JsFuture::from(cancel).map(|_x| ()),
    ).await.map_err(stringify)?;

    Ok("".into())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReceiveResult {
    data: Vec<u8>,
    filename: String,
    filesize: u64,
}

fn stringify(e: impl Error) -> String { format!("error code: {}", e) }

#[wasm_bindgen]
pub async fn receive(config: WormholeConfig, code: String, cancel: js_sys::Promise, progress_handler: js_sys::Function) -> Result<JsValue, JsValue> {
    let event_handler = Rc::new(Box::new(move |e: event::Event| {
        progress_handler.call1(&JsValue::null(), &JsValue::from_serde(&e).unwrap()).expect("progress_handler call should succeed");
    }) as Box<dyn Fn(event::Event)>);

    let (server_welcome, wormhole) = Wormhole::connect_with_code(
        transfer::APP_CONFIG.rendezvous_url(config.rendezvous_url.into()),
        Code(code),
    ).await.map_err(stringify)?;

    event_handler(event::server_welcome(server_welcome.welcome.unwrap_or_default()));

    let req = transfer::request_file(
        wormhole,
        url::Url::parse(&config.relay_url).unwrap(),
        transit::Abilities::FORCE_RELAY,
        wasm_bindgen_futures::JsFuture::from(cancel.clone()).map(|_x| ()),
    ).await.map_err(stringify)?.ok_or("")?;

    let filename = req.filename.to_str().unwrap_or_default().to_string();
    let filesize = req.filesize;
    event_handler(event::file_metadata(filename.clone(), filesize));

    let ph = event_handler.clone();
    let mut file: Vec<u8> = Vec::new();
    req.accept(
        |_info, _address| {
            event_handler(event::connected());
        },
        move |cur, total| {
            ph(event::progress(cur, total));
        },
        &mut file,
        wasm_bindgen_futures::JsFuture::from(cancel).map(|_x| ()),
    ).await.map_err(stringify)?;

    let result = ReceiveResult {
        data: file,
        filename,
        filesize,
    };
    Ok(JsValue::from_serde(&result).unwrap())
}
