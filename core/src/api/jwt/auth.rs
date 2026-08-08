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
use crate::api::control::ApiState;
use crate::api::error::Error;
use crate::api::jwt::generate::{
    AccessTokenClaims, RefreshTokenClaims, generate_access_token, generate_refresh_token,
};
use crate::orm::error::Error::Unauthorized;
use crate::orm::tunnel_user::authenticate_tunnel_user;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Json, http};
use jsonwebtoken::{Algorithm, Validation, get_current_timestamp};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{debug, info, info_span};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}
pub async fn login(
    State(api_state): State<Arc<ApiState>>,
    Json(body): Json<LoginBody>,
) -> Result<Response<Body>, Error> {
    let (id, is_admin) = authenticate_tunnel_user(
        api_state.shared.db_connection.clone(),
        api_state.shared.auth_manager.clone(),
        body.username.as_str(),
        body.password.as_str(),
    )
    .await?;

    if !is_admin {
        Err(Unauthorized)?
    }

    let (jti, sid, refresh_token) =
        generate_refresh_token(id.clone(), &api_state.refresh_token_keys.encoding_key)?;
    let access_token = generate_access_token(
        id.clone(),
        sid.clone(),
        &api_state.access_token_keys.encoding_key,
    )?;

    api_state.jti_map.retain(|_, value| value.0 != id);

    api_state.jti_map.insert(
        jti,
        (
            id.clone(),
            sid,
            Instant::now() + Duration::from_secs(24 * 60 * 60),
        ),
    );

    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::new(
        json!({
            "refresh_token": refresh_token,
            "access_token": access_token
        })
        .to_string(),
    );

    let api_span = info_span!("api", user_id = %id);
    api_span.in_scope(|| info!("Api user logged in"));

    Ok(response_builder.body(response_body)?)
}

#[derive(serde::Deserialize)]
pub struct RefreshTokenBody {
    refresh_token: String,
}
pub async fn refresh_token(
    State(api_state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(refresh_token_body): Json<RefreshTokenBody>,
) -> Result<Response<Body>, Error> {
    let mut refresh_validation = Validation::new(Algorithm::RS256);
    refresh_validation.set_required_spec_claims(&["sub", "iat", "exp"]);
    let refresh_token_data = jsonwebtoken::decode::<RefreshTokenClaims>(
        refresh_token_body.refresh_token,
        &api_state.refresh_token_keys.decoding_key,
        &refresh_validation,
    )
    .map_err(|_| Unauthorized)?;

    let access_token = headers.get("Authorization").ok_or(Unauthorized)?;
    let access_token_data = jsonwebtoken::decode::<AccessTokenClaims>(
        access_token.to_str().map_err(|_| Unauthorized)?,
        &api_state.access_token_keys.decoding_key,
        &refresh_validation,
    )
    .map_err(|_| Unauthorized)?;

    if refresh_token_data.claims.sub != access_token_data.claims.sub
        || access_token_data.claims.exp < get_current_timestamp()
    {
        Err(Unauthorized)?
    }

    let entry = api_state
        .jti_map
        .get(&refresh_token_data.claims.jti)
        .ok_or(Unauthorized)?;

    let access_token = generate_access_token(
        entry.value().0.clone(),
        entry.value().1.clone(),
        &api_state.access_token_keys.encoding_key,
    )?;

    let response_body = Body::new(
        json!({
            "access_token": access_token
        })
        .to_string(),
    );

    let api_span = info_span!("api", user_id = %refresh_token_data.claims.sub);
    api_span.in_scope(|| debug!("Api user token refreshed"));

    Ok(Response::new(response_body))
}

pub async fn logout(
    State(api_state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let access = headers.get("Authorization").ok_or(Unauthorized)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_required_spec_claims(&["sub", "iat", "exp"]);
    let claims = jsonwebtoken::decode::<AccessTokenClaims>(
        access,
        &api_state.access_token_keys.decoding_key,
        &validation,
    )
    .map_err(|_| Unauthorized)?;

    api_state
        .jti_map
        .retain(|_, value| value.1 != claims.claims.sid);

    let api_span = info_span!("api", user_id = %claims.claims.sub);
    api_span.in_scope(|| info!("Api user logged out"));

    Ok(StatusCode::OK.into_response())
}

pub async fn access_token_middleware(
    State(api_state): State<Arc<ApiState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let access = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_required_spec_claims(&["sub", "iat", "exp"]);
    jsonwebtoken::decode::<AccessTokenClaims>(
        access,
        &api_state.access_token_keys.decoding_key,
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}
