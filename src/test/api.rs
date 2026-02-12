use crate::*;
use api::{
    AUTHORIZATION, Body, EngineErrorResponse, HttpRequest, HttpResponse, Router, StatusCode,
    signature_header,
};
use de::DeserializeOwned;
use key::Signer;
use tower_service::Service;

pub struct ApiClient {
    router: Router,
    signer: Signer,
}

impl ApiClient {
    pub async fn new(router: Router) -> Result<Self> {
        let signer = Signer::new()?;

        let mut client = Self { router, signer };
        let _: () = client.get("/health", json::value!({})).await?;
        let _: () = client
            .post(
                "/health",
                json::value!({
                    "request_id": Id::generate().await
                }),
            )
            .await?;

        Ok(client)
    }
    pub async fn get<R>(&mut self, uri: &'static str, json_value: json::Value) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.request("GET", uri, json_value).await
    }
    pub async fn get_body(&mut self, uri: &'static str, json_value: json::Value) -> Result<Body> {
        self.request_body("GET", uri, json_value).await
    }
    pub async fn post<R>(&mut self, uri: &'static str, json_value: json::Value) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.request("POST", uri, json_value).await
    }
    pub async fn post_body(&mut self, uri: &'static str, json_value: json::Value) -> Result<Body> {
        self.request_body("POST", uri, json_value).await
    }
    async fn request_http_response(
        &mut self,
        method: &'static str,
        uri: &'static str,
        mut json_value: json::Value,
    ) -> Result<HttpResponse> {
        json_value["timestamp"] = json::value!(time::now());
        let body_bytes = json::to_vec(&json_value)?;

        let signature_base64 = self.signer.sign(&body_bytes)?;

        let request = HttpRequest::builder()
            .method(method)
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, signature_header(signature_base64))
            .uri(uri)
            .body(Body::from(body_bytes))?;

        let response = self.router.call(request).await?;
        if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
            let response: EngineErrorResponse = json::from_body(response.into_body()).await?;
            return Err(Error::EngineError(response.code));
        }
        if response.status() != StatusCode::OK {
            let response_status = response.status();
            let response_value: json::Value = json::from_body(response.into_body()).await?;
            if let Some(message) = response_value.get("message").and_then(|v| v.as_str()) {
                return Unexpected!("response status {}: {}", response_status, message);
            }
            return Unexpected!("response status {}: {}", response_status, response_value);
        }
        Ok(response)
    }
    pub async fn request_body(
        &mut self,
        method: &'static str,
        uri: &'static str,
        json_value: json::Value,
    ) -> Result<Body> {
        let http_response = self.request_http_response(method, uri, json_value).await?;
        Ok(http_response.into_body())
    }
    pub async fn request<R>(
        &mut self,
        method: &'static str,
        uri: &'static str,
        json_value: json::Value,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let http_response = self.request_http_response(method, uri, json_value).await?;
        return Ok(json::from_body(http_response.into_body()).await?);
    }
}
