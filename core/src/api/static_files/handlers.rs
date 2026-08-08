/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::api::error::Error;
use axum::body::Body;
use axum::extract::Path;
use axum::http;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../webui/dist"]
struct Webui;

pub async fn index_handler() -> Result<Response<Body>, Error> {
    static_file_handler(Path("index.html".to_string())).await
}
pub async fn static_file_handler(path: Path<String>) -> Result<Response<Body>, Error> {
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };

    let (path, file) = match Webui::get(path) {
        Some(file) => (path, file),
        None if !path.starts_with("assets/") => match Webui::get("index.html") {
            Some(file) => ("index.html", file),
            None => {
                return Ok(StatusCode::NOT_FOUND.into_response());
            }
        },
        None => {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }
    };

    let content_type = mime_guess::from_path(path).first_or_octet_stream();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, content_type.as_ref())
        .body(Body::from(file.data))?)
}
