use eyre::eyre;
use http::HeaderMap;
use reqwest::Client;

use crate::api::endpoint::Endpoint;

pub async fn http_urlencoded_query_request<E: Endpoint>(
    base_url: &str,
    body: &E::Request,
    headers: HeaderMap,
) -> eyre::Result<E::Response> {
    let data = serde_urlencoded::to_string(body)?;
    let url = format!("{}{}?{}", base_url, E::PATH, data);
    let response = Client::new()
        .request(E::METHOD, &url)
        .headers(headers)
        .send()
        .await?;

    let response = response
        .text()
        .await?;

    let response = serde_json::from_str(&response)
        .map_err(|err| eyre!("{err}, url: {url}, response: {response}"))?;
    Ok(response)
}
