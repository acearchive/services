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
    let upstream_url = format!(
        "{}/media{}",
        data_connector_url.trim_end_matches('/'),
        request_path
    );

    let mut upstream_request = Request::new(&upstream_url, req.method())?;
    let upstream_request_headers = upstream_request.headers_mut()?;
    *upstream_request_headers = req.headers().clone();

    let mut response = Fetch::Request(upstream_request).send().await?;
    let response_headers = response.headers_mut();
    response_headers.set("Cache-Control", &format!("public, max-age={cache_ttl}"))?;

    Ok(response)
}
