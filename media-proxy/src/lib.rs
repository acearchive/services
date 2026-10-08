use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let data_connector_url = env.var("DATA_CONNECTOR_URL")?.to_string();
    let cache_ttl = env.var("CACHE_TTL")?.to_string();

    if req.method() != Method::Get && req.method() != Method::Head {
        return Ok(Response::empty()?.with_status(405));
    }

    let request_url = req.url()?;
    let request_path = request_url.path();

    if request_path.starts_with("/assets/") {
        let asset_url = data_connector_url.trim_end_matches('/').to_string() + request_path;
        return Fetch::Url(asset_url.parse()?).send().await;
    }

    // The upstream Omeka Data Connector would already handle this case fine, but short-circuiting
    // here saves us the trip.
    if !request_path.starts_with("/artifacts/")
        && !request_path.starts_with("/a/")
        && !request_path.starts_with("/r/")
    {
        return Ok(Response::empty()?.with_status(404));
    }

    let upstream_url = format!(
        "{}/media{}",
        data_connector_url.trim_end_matches('/'),
        request_path
    );

    let upstream_request = Request::new_with_init(
        &upstream_url,
        &RequestInit {
            body: None,
            headers: req.headers().clone(),
            cf: Default::default(),
            method: req.method(),
            // We want to pass through redirects to the client.
            redirect: RequestRedirect::Manual,
            cache: None,
        },
    )?;

    let mut response = Fetch::Request(upstream_request).send().await?.cloned()?;
    let response_headers = response.headers_mut();
    response_headers.set("Cache-Control", &format!("public, max-age={cache_ttl}"))?;

    Ok(response)
}
