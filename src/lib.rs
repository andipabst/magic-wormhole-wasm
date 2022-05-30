use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use magic_wormhole::{Code, transfer, transit, Wormhole};
use wasm_bindgen::prelude::*;
use log::Level;

mod event;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

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

struct NoOpFuture {}

impl Future for NoOpFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

#[wasm_bindgen]
pub async fn send(config: WormholeConfig, file_input: web_sys::HtmlInputElement, progress_handler: js_sys::Function) -> Result<JsValue, JsValue> {
    let this = JsValue::null();

    let file_list = file_input
        .files()
        .expect("Failed to get filelist from File Input!");
    if file_list.length() < 1 || file_list.get(0) == None {
        return Err("Please select at least one valid file.".into());
    }

    let file: web_sys::File = file_list.get(0).expect("Failed to get File from filelist!");
    let file_content = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await?;
    let array = js_sys::Uint8Array::new(&file_content);
    let len = array.byte_length() as u64;
    let data_to_send: Vec<u8> = array.to_vec();

    let (server_welcome, connector) = Wormhole::connect_without_code(
        transfer::APP_CONFIG.rendezvous_url(config.rendezvous_url.into()),
        2,
    ).await.map_err(stringify)?;

    progress_handler.call1(&this, &event::code(server_welcome.code.0))?;

    let wormhole = connector.await.map_err(stringify)?;
    transfer::send_file(
        wormhole,
        url::Url::parse(&config.relay_url).unwrap(),
        &mut &data_to_send[..],
        PathBuf::from(file.name()),
        len,
        transit::Abilities::FORCE_RELAY,
        |_info, _address| {
            progress_handler.call1(&this, &event::connected());
        },
        |cur, total| {
            // TODO send progress via the progress handler
            console_log!("Progress: {}/{}", cur, total);
        },
        NoOpFuture {},
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
pub async fn receive(config: WormholeConfig, code: String, progress_handler: js_sys::Function) -> Result<JsValue, JsValue> {
    let this = JsValue::null();

    let (server_welcome, wormhole) = Wormhole::connect_with_code(
        transfer::APP_CONFIG.rendezvous_url(config.rendezvous_url.into()),
        Code(code),
    ).await.map_err(stringify)?;

    progress_handler.call1(&this, &event::server_welcome(server_welcome.welcome.unwrap_or_default()))?;

    let req = transfer::request_file(
        wormhole,
        url::Url::parse(&config.relay_url).unwrap(),
        transit::Abilities::FORCE_RELAY,
        NoOpFuture {},
    ).await.map_err(stringify)?.ok_or("")?;

    let filename = req.filename.to_str().unwrap_or_default().to_string();
    let filesize = req.filesize;
    progress_handler.call1(&this, &event::file_metadata(filename.clone(), filesize))?;

    let mut file: Vec<u8> = Vec::new();
    req.accept(
        |_info, _address| {
            progress_handler.call1(&this, &event::connected());
        },
        |cur, total| {
            // TODO send progress via the progress handler
            console_log!("Progress: {}/{}", cur, total);
        },
        &mut file,
        NoOpFuture {},
    ).await.map_err(stringify)?;

    let result = ReceiveResult {
        data: file,
        filename,
        filesize,
    };
    Ok(JsValue::from_serde(&result).unwrap())
}
