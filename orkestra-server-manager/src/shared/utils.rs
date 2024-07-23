#![allow(unused)]

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub fn ok<T>(value: T) -> (StatusCode, Json<serde_json::Value>)
where
    T: Serialize,
{
    (StatusCode::OK, Json(serde_json::to_value(value).unwrap()))
}

pub fn ok_json(value: serde_json::Value) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(value))
}

pub fn just_ok() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({})))
}

pub fn created<T>(value: T) -> (StatusCode, Json<serde_json::Value>)
where
    T: Serialize,
{
    (
        StatusCode::CREATED,
        Json(serde_json::to_value(value).unwrap()),
    )
}

pub fn created_json(value: serde_json::Value) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::CREATED, Json(value))
}

pub fn just_created() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::CREATED, Json(serde_json::json!({})))
}

pub fn bad_request<T>(value: T) -> (StatusCode, Json<serde_json::Value>)
where
    T: Serialize,
{
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::to_value(value).unwrap()),
    )
}

pub fn bad_request_json(value: serde_json::Value) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(value))
}

pub fn just_bad_request() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({})))
}

pub fn unauthorized<T>(value: T) -> (StatusCode, Json<serde_json::Value>)
where
    T: Serialize,
{
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::to_value(value).unwrap()),
    )
}

pub fn unauthorized_json(value: serde_json::Value) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::UNAUTHORIZED, Json(value))
}

pub fn just_unauthorized() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({})))
}
