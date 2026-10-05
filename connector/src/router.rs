use std::fmt;

use axum::{
    Json, Router,
    body::Body,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use super::{
    models::Item,
    omeka,
    resolver::{CanonicalUrl, MediaLocator, Resolver},
};

pub fn new() -> Router {
    Router::new()
        .route("/health", get(get_health))
        .route("/media/artifacts/{slug}/{filename}", get(get_media_long))
        .route("/media/a/{id}/{filename}", get(get_media_short))
        .route("/media/r/{id}/{filename}", get(get_media_raw))
        .route("/items", get(get_items))
}

fn map_error<E>(code: StatusCode) -> impl FnOnce(E) -> StatusCode
where
    E: fmt::Display,
{
    move |err| {
        log::error!("{}", err);
        code
    }
}

fn proxy_response(response: reqwest::Response) -> Response {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes_stream();

    let mut axum_response = Response::builder().status(status);
    *axum_response.headers_mut().unwrap() = headers;
    axum_response.body(Body::from_stream(body)).unwrap()
}

async fn get_media(locator: MediaLocator) -> Result<impl IntoResponse, StatusCode> {
    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);
    let location = resolver
        .resolve_media(locator)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    match location {
        Some(location) => match location.canonical_url {
            CanonicalUrl::ShouldRedirect(url) => {
                Ok(Redirect::permanent(url.as_str()).into_response())
            }
            CanonicalUrl::AlreadyCanonical => Ok(proxy_response(
                client
                    .get(location.omeka_url)
                    .send()
                    .await
                    .map_err(map_error(StatusCode::BAD_GATEWAY))?,
            )),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[axum::debug_handler]
async fn get_health() -> StatusCode {
    StatusCode::OK
}

#[axum::debug_handler]
async fn get_media_long(
    Path((slug, filename)): Path<(omeka::AceSlug, omeka::AceFilename)>,
) -> Result<impl IntoResponse, StatusCode> {
    get_media(MediaLocator::Long { slug, filename }).await
}

#[axum::debug_handler]
async fn get_media_short(
    Path((id, filename)): Path<(omeka::AceId, omeka::AceFilename)>,
) -> Result<impl IntoResponse, StatusCode> {
    get_media(MediaLocator::Short { id, filename }).await
}

#[axum::debug_handler]
async fn get_media_raw(
    Path((id, filename)): Path<(omeka::AceId, omeka::AceFilename)>,
) -> Result<impl IntoResponse, StatusCode> {
    get_media(MediaLocator::Raw { id, filename }).await
}

#[axum::debug_handler]
async fn get_items() -> Result<Json<Vec<Item>>, StatusCode> {
    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);

    Ok(Json(
        resolver
            .list_items()
            .await
            .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?,
    ))
}
