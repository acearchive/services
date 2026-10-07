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
    assets::render_css,
    assets::{FilePage, FilePageContext},
    config,
    models::{CollectionMetadata, Item},
    omeka,
    resolver::{CanonicalUrl, MediaLocator, Resolver},
};

pub fn new() -> Router {
    Router::new()
        .route("/health", get(get_health))
        .route("/media/artifacts/{slug}/{filename}", get(get_media_long))
        .route("/media/a/{id}/{filename}", get(get_media_short))
        .route("/media/r/{id}/{filename}", get(get_media_raw))
        .route("/hugo-items", get(get_hugo_items))
        .route("/hugo-metadata", get(get_hugo_metadata))
        .route("/assets/style.css", get(get_asset_style))
        .route("/assets/script.js", get(get_asset_script))
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
    let is_raw = matches!(locator, MediaLocator::Raw { .. });

    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);
    let location = resolver.resolve_media(locator).await;

    match location {
        Ok(location) => match location.canonical_url {
            CanonicalUrl::ShouldRedirect(url) => {
                Ok(Redirect::permanent(url.as_str()).into_response())
            }
            CanonicalUrl::AlreadyCanonical => {
                if is_raw {
                    Ok(proxy_response(
                        client
                            .get(location.omeka_url)
                            .send()
                            .await
                            .map_err(map_error(StatusCode::BAD_GATEWAY))?,
                    ))
                } else {
                    let file_page_context = FilePageContext::from_location(&location)
                        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

                    FilePage::render(&file_page_context)
                        .map(|page| {
                            (
                                StatusCode::OK,
                                [("Content-Type", "text/html")],
                                page.to_string(),
                            )
                                .into_response()
                        })
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        Err(err) if err.downcast_ref::<omeka::SkipError>().is_some() => Err(StatusCode::NOT_FOUND),
        Err(err) => Err(map_error(StatusCode::INTERNAL_SERVER_ERROR)(err)),
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
async fn get_hugo_items() -> Result<Json<Vec<Item>>, StatusCode> {
    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);

    Ok(Json(
        resolver
            .list_all_items()
            .await
            .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?,
    ))
}

#[axum::debug_handler]
async fn get_hugo_metadata() -> Result<Json<CollectionMetadata>, StatusCode> {
    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);

    Ok(Json(CollectionMetadata {
        tags: resolver
            .list_all_tags()
            .await
            .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?,
    }))
}

#[axum::debug_handler]
async fn get_asset_style() -> Result<impl IntoResponse, StatusCode> {
    let base_domain =
        config::base_domain().map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    let css = render_css(&base_domain);

    Ok((StatusCode::OK, [("Content-Type", "text/css")], css))
}

#[axum::debug_handler]
async fn get_asset_script() -> impl IntoResponse {
    let js = include_str!("assets/script.js");
    (StatusCode::OK, [("Content-Type", "text/javascript")], js)
}
