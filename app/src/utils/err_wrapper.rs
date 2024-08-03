use std::{convert::Infallible, fmt::Display};

use axum::{http::StatusCode, response::IntoResponse};

pub enum ErrWrapper {
    Worker(worker::Error),
    Axum(axum::http::Error),
    Json(serde_json::Error),
    JsValue(wasm_bindgen::JsValue),
    Raw(String),
}

impl From<worker::Error> for ErrWrapper {
    fn from(value: worker::Error) -> Self {
        ErrWrapper::Worker(value)
    }
}

impl From<axum::http::Error> for ErrWrapper {
    fn from(value: axum::http::Error) -> Self {
        ErrWrapper::Axum(value)
    }
}

impl From<serde_json::Error> for ErrWrapper {
    fn from(value: serde_json::Error) -> Self {
        ErrWrapper::Json(value)
    }
}

impl From<String> for ErrWrapper {
    fn from(value: String) -> Self {
        ErrWrapper::Raw(value)
    }
}

impl From<wasm_bindgen::JsValue> for ErrWrapper {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        ErrWrapper::JsValue(value)
    }
}

impl From<Infallible> for ErrWrapper {
    fn from(_: Infallible) -> Self {
        unimplemented!()
    }
}

impl Display for ErrWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrWrapper::Worker(err) => err.fmt(f),
            ErrWrapper::Axum(err) => err.fmt(f),
            ErrWrapper::Json(err) => err.fmt(f),
            ErrWrapper::Raw(err) => err.fmt(f),
            ErrWrapper::JsValue(_) => write!(f, "Err(JsValue)"),
        }
    }
}

impl IntoResponse for ErrWrapper {
    fn into_response(self) -> axum::response::Response {
        match self {
            ErrWrapper::Worker(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Axum(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Json(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            ErrWrapper::Raw(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
            ErrWrapper::JsValue(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unknown JsValue Error").into_response()
            }
        }
    }
}
