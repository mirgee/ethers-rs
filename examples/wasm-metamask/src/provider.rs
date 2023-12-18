use async_trait::async_trait;
use ethers::providers::{
    JsonRpcClient, JsonRpcError, ProviderError as EthersProviderError, RpcError,
};
use js_sys::{Function, Promise, Reflect};
use serde::{
    de::{DeserializeOwned, Error},
    Serialize,
};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

#[derive(thiserror::Error, Debug)]
pub enum EIP1193Error {
    #[error("rcp error: {0}")]
    RPC(JsonRpcError),
    #[error("deserialize error: {0}")]
    Deserialize(serde_json::Error),
    #[error("JS value error: {0}")]
    JsValueError(String),
}

impl From<JsValue> for EIP1193Error {
    fn from(js: JsValue) -> Self {
        Self::JsValueError(format!("{:?}", js))
    }
}

impl From<EIP1193Error> for EthersProviderError {
    fn from(err: EIP1193Error) -> Self {
        match err {
            EIP1193Error::RPC(_) => EthersProviderError::UnsupportedRPC,
            EIP1193Error::Deserialize(err) => EthersProviderError::SerdeJson(err),
            EIP1193Error::JsValueError(some_string) => {
                EthersProviderError::CustomError(some_string)
            }
        }
    }
}

impl RpcError for EIP1193Error {
    fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
        if let EIP1193Error::RPC(err) = self {
            Some(err)
        } else {
            None
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        if let EIP1193Error::Deserialize(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EIP1193 {
    this: JsValue,
    request: Function,
    on: Function,
    remove_listener: Function,
}

// TODO: Implement a threadsafe solution
// for now, we will just use single thread in WASM context
unsafe impl Send for EIP1193 {}
unsafe impl Sync for EIP1193 {}

impl EIP1193 {
    pub fn new(win: &Window) -> Result<Self, EIP1193Error> {
        let provider =
            win.get("ethereum").ok_or(EIP1193Error::JsValueError("missing provider".to_owned()))?;
        Ok(Self {
            request: Reflect::get(&provider, &JsValue::from("request"))?.into(),
            on: Reflect::get(&provider, &JsValue::from("on"))?.into(),
            remove_listener: Reflect::get(&provider, &JsValue::from("removeListener"))?.into(),
            this: provider.into(),
        })
    }
}

#[derive(Serialize, Debug)]
struct RequestMethod<T: Serialize + Debug> {
    pub method: String,
    pub params: Option<T>,
}

fn parse_js<T: for<'de> serde::Deserialize<'de>>(data: JsValue) -> Result<T, EIP1193Error> {
    serde_wasm_bindgen::from_value(data).map_err(|err| {
        EIP1193Error::Deserialize(serde_json::Error::custom(&format!(
            "failed to parse js value: {:?}",
            err
        )))
    })
}

fn to_deserialize_error(err: impl Error) -> EIP1193Error {
    EIP1193Error::Deserialize(serde_json::Error::custom(err.to_string()))
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for EIP1193 {
    type Error = EIP1193Error;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        web_sys::console::log_1(
            &format!("request method {:?}, params: {:?}", method, params).into(),
        );
        let arg = RequestMethod { method: method.to_string(), params: Some(params) };
        let promise = self.request.call1(
            &self.this,
            &serde_wasm_bindgen::to_value(&arg).map_err(to_deserialize_error)?,
        )?;
        let parsed = parse_js(JsFuture::from(Promise::from(promise)).await?)?;
        Ok(parsed)
    }
}
