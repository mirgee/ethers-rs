use crate::provider::EIP1193;
use ethers::{contract::abigen, prelude::Provider, providers::Middleware};
use wasm_bindgen::prelude::*;
use web_sys::console;

pub mod provider;
pub mod utils;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

abigen!(SimpleContract, "./abi/contract.json", derives(serde::Deserialize, serde::Serialize));

#[wasm_bindgen]
pub async fn deploy() {
    utils::set_panic_hook();

    console::log_2(
        &"SimpleContract ABI: ".into(),
        &serde_wasm_bindgen::to_value(&*SIMPLECONTRACT_ABI).unwrap(),
    );

    let window = web_sys::window().unwrap();
    let transport = EIP1193::new(&window).unwrap();
    let provider = Provider::<EIP1193>::new(transport);
    log!("Metmask provider created, obtaining accounts...");
    let accounts = provider.get_accounts().await.unwrap();
    log!("Obtain accounts: {:?}", accounts);
}
