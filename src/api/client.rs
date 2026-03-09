use super::{AUTHORIZATION, Body, Method, Protocol, Request, StatusCode, TRACE, signature_header};
use crate::*;
use key::Signer;

#[derive(Clone)]
pub struct Client {
    host: config::Host,
    client: http::Client,
    signer: Signer,
}

#[derive(Deserialize, Debug)]
pub struct EngineErrorResponse {
    pub code: i64,
}

impl Client {
    // TODO 传递context，使得verify 不用初始化context
    pub async fn new(host: config::Host) -> Result<Self> {
        let signer = Signer::new()?;
        let api_client = Self {
            host: host.clone(),
            client: http::Client::new(),
            signer: signer,
        };

        if let config::Host::NotSet = host {
            Ok(api_client)
        } else {
            api_client.verify().await?;
            Ok(api_client)
        }
    }

    pub async fn get<T, P>(&self, context: &Context, path: &'static str, request: T) -> Result<P>
    where
        T: Protocol,
        P: Protocol,
    {
        self.request(context, Method::GET, path, request).await
    }

    pub async fn get_body<T>(
        &self,
        context: &Context,
        path: &'static str,
        request: T,
    ) -> Result<Body>
    where
        T: Protocol,
    {
        self.request_body(context, Method::GET, path, request).await
    }

    pub async fn post<T, P>(&self, context: &Context, path: &'static str, request: T) -> Result<P>
    where
        T: Protocol,
        P: Protocol,
    {
        self.request(context, Method::POST, path, request).await
    }

    pub async fn post_body<T, P>(
        &self,
        context: &Context,
        path: &'static str,
        request: T,
    ) -> Result<Body>
    where
        T: Protocol,
    {
        self.request_body(context, Method::POST, path, request)
            .await
    }

    async fn request_response<T>(
        &self,
        context: &Context,
        method: Method,
        path: &'static str,
        request: T,
    ) -> Result<http::Response>
    where
        T: Protocol,
    {
        if let config::Host::NotSet = self.host {
            return Unexpected!("host not set");
        }
        if context.in_transaction().await {
            return Unexpected!("request in transaction");
        }

        let data_json = json::to_string(&request)?;

        // TODO support context.cancel
        // TODO add request log

        let signature_base64 = self.signer.sign(data_json.as_bytes())?;
        let trace_id: i64 = context.trace_id.into();
        let response = self
            .client
            .request(method.clone(), self.host.for_client(path)?)
            .header("Content-Type", "application/json")
            .header(TRACE, trace_id)
            .header(AUTHORIZATION, signature_header(signature_base64))
            .body(data_json)
            .send()
            .await?;

        let status_code = response.status();
        if status_code == StatusCode::UNPROCESSABLE_ENTITY {
            let engine_error_response: EngineErrorResponse =
                json::from_str(&response.text().await?)?;
            return Err(Error::EngineError(engine_error_response.code));
        }
        if status_code != StatusCode::OK {
            return Unexpected!(
                "status code {:?} when {:?} {:?} {:?}: {}",
                status_code,
                method,
                self.host,
                path,
                response.text().await?
            );
        }
        Ok(response)
    }

    pub async fn request<T, P>(
        &self,
        context: &Context,
        method: Method,
        path: &'static str,
        request: T,
    ) -> Result<P>
    where
        T: Protocol,
        P: Protocol,
    {
        let response = self
            .request_response(context, method, path, request)
            .await?;
        return Ok(json::from_str(&response.text().await?)?);
    }

    pub async fn request_body<T>(
        &self,
        context: &Context,
        method: Method,
        path: &'static str,
        request: T,
    ) -> Result<Body>
    where
        T: Protocol,
    {
        let response = self
            .request_response(context, method, path, request)
            .await?;

        Ok(Body::from_stream(response.bytes_stream()))
    }

    async fn verify(&self) -> Result<()> {
        #[cfg(feature = "cloud")]
        let context = &Context::background(None);
        #[cfg(not(feature = "cloud"))]
        let context = &Context::background();
        let _: () = self.get(context, "/health", Request::EMPTY).await?;
        let _: () = self
            .post(context, "/health", Request::new(Id::generate().await))
            .await?;
        Ok(())
    }
}
