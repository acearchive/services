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
    hugo::{CollectionMetadata, Item, Tag},
    omeka,
    rdf::{ModelOptions, ModelSerializer},
    resolver::{CanonicalUrl, MediaLocator, Resolver},
};

pub fn new() -> Router {
    Router::new()
        .route("/health", get(get_health))
        .route("/media/artifacts/{slug}/{*filename}", get(get_media_long))
        .route("/media/a/{id}/{*filename}", get(get_media_short))
        .route("/media/r/{id}/{*filename}", get(get_media_raw))
        .route("/hugo-items", get(get_hugo_items))
        .route("/hugo-metadata", get(get_hugo_metadata))
        .route("/jsonld", get(get_jsonld))
        .route("/jsonld/items/{id}", get(get_jsonld_item))
        .route("/jsonld/files/{id}/{*filename}", get(get_jsonld_file))
        .route("/jsonld/people/{name}", get(get_jsonld_person))
        .route("/jsonld/identities/{name}", get(get_jsonld_identity))
        .route("/jsonld/collections/{name}", get(get_jsonld_collection))
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

fn new_resolver() -> Result<Resolver, StatusCode> {
    let omeka_client = omeka::Client::from_config(reqwest::Client::new())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Resolver::new(omeka_client))
}

fn jsonld_response(serializer: ModelSerializer) -> Result<impl IntoResponse, StatusCode> {
    Ok((
        [("Content-Type", "application/ld+json")],
        Json(
            serializer
                .into_json()
                .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?,
        ),
    ))
}

// Documents for individual entities are intended to be embedded in public web pages, so they omit
// project-specific terms.
fn entity_serializer() -> ModelSerializer {
    ModelSerializer::new(ModelOptions {
        include_private_terms: false,
    })
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

                    match FilePage::render(&file_page_context) {
                        Some(page) => Ok((
                            StatusCode::OK,
                            [("Content-Type", "text/html")],
                            page.to_string(),
                        )
                            .into_response()),
                        None => Ok(proxy_response(
                            client
                                .get(location.omeka_url)
                                .send()
                                .await
                                .map_err(map_error(StatusCode::BAD_GATEWAY))?,
                        )),
                    }
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

    let items = resolver
        .list_all_items()
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(
        items
            .iter()
            .cloned()
            .map(Item::try_from)
            .collect::<anyhow::Result<Vec<_>>>()
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
            .list_all_collections()
            .await
            .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
            .iter()
            .cloned()
            .map(Tag::from)
            .collect(),
    }))
}

#[axum::debug_handler]
async fn get_jsonld() -> Result<impl IntoResponse, StatusCode> {
    let client = reqwest::Client::new();
    let omeka_client = omeka::Client::from_config(client.clone())
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
    let resolver = Resolver::new(omeka_client);

    let items = resolver
        .list_all_items()
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut serializer = ModelSerializer::new(ModelOptions {
        include_private_terms: true,
    });

    serializer
        .add_items(&items)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_jsonld_item(Path(id): Path<omeka::AceId>) -> Result<impl IntoResponse, StatusCode> {
    let item = new_resolver()?
        .find_item(&id)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut serializer = entity_serializer();

    serializer
        .add_item(&item)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_jsonld_file(
    Path((id, filename)): Path<(omeka::AceId, omeka::AceFilename)>,
) -> Result<impl IntoResponse, StatusCode> {
    let file = new_resolver()?
        .find_file(&id, &filename)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut serializer = entity_serializer();

    serializer
        .add_file(&file, &id)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_jsonld_person(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let (person, items) = new_resolver()?
        .find_person(&name)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut serializer = entity_serializer();

    serializer
        .add_person_document(&person, &items)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_jsonld_identity(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let (identity, items) = new_resolver()?
        .find_identity(&name)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut serializer = entity_serializer();

    serializer
        .add_identity_document(&identity, &items)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_jsonld_collection(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let (collection, items) = new_resolver()?
        .find_collection(&name)
        .await
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut serializer = entity_serializer();

    serializer
        .add_collection_document(&collection, &items)
        .map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    jsonld_response(serializer)
}

#[axum::debug_handler]
async fn get_asset_style() -> Result<impl IntoResponse, StatusCode> {
    let base_domain =
        config::base_domain().map_err(map_error(StatusCode::INTERNAL_SERVER_ERROR))?;

    let css = render_css(&base_domain);

    Ok((
        StatusCode::OK,
        [("Content-Type", "text/css"), ("Cache-Control", "no-cache")],
        css,
    ))
}

#[axum::debug_handler]
async fn get_asset_script() -> impl IntoResponse {
    let js = include_str!("assets/script.js");
    (
        StatusCode::OK,
        [
            ("Content-Type", "text/javascript"),
            ("Cache-Control", "no-cache"),
        ],
        js,
    )
}
