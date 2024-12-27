/// # IPQuery Actix Web Middleware
///
/// This middleware allows you to query IP information using the `ipapi` crate and store the results
/// using a custom store that implements the `IPQueryStore` trait. It supports querying the IP address
/// from either the `X-Forwarded-For` header or the peer address of the request.
///
/// ## Features
/// - Query IP information using a specified endpoint.
/// - Store IP information using a custom store.
/// - Option to use the `X-Forwarded-For` header for IP address extraction.
///
/// ## Usage Example
/// ```rust
/// use actix_web::{web, App, HttpServer};
/// use actix_ipquery::{IPQuery, IPQueryStore};
///
/// #[derive(Clone)]
/// struct MyStore;
///
/// impl IPQueryStore for MyStore {
///     fn store(&self, ip_info: ipapi::IPInfo) -> Result<(), std::io::Error> {
///         println!("{:?}", ip_info);
///         Ok(())
///     }
/// }
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(IPQuery::new(MyStore).finish())
///             .route("/", web::get().to(|| async { "Hello, world!" }))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
use ipapi::{query_ip_with_endpoint, Error as ReqwestError, IPInfo};
use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

/// The IPQuery struct that implements actix-web's middleware.
#[derive(Clone)]
pub struct IPQuery<T: IPQueryStore> {
    endpoint: String,
    store: T,
    forwarded_for: bool,
}
impl<T: IPQueryStore> IPQuery<T> {
    /// Create a new IPQuery middleware
    pub fn new(store: T) -> IPQuery<T> {
        IPQuery {
            endpoint: "https://api.ipquery.io/".to_owned(),
            store,
            forwarded_for: false,
        }
    }
    /// Set the endpoint for the IP query
    pub fn endpoint(&mut self, endpoint: &str) -> &mut Self {
        self.endpoint = endpoint.to_owned();
        self
    }
    /// Use the `X-Forwarded-For` header for IP address extraction
    pub fn forwarded_for(&mut self, y: bool) -> &mut Self {
        self.forwarded_for = y;
        self
    }
    /// Finish the configuration and return the middleware
    pub fn finish(&self) -> IPQuery<T> {
        self.clone()
    }
    async fn query_ip(&self, ip: &str) -> Result<IPInfo, ReqwestError> {
        query_ip_with_endpoint(ip, &self.endpoint).await
    }
}
impl<S, B, T> Transform<S, ServiceRequest> for IPQuery<T>
where
    T: IPQueryStore + 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: IPQueryStore + Clone,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = IPQueryMiddleware<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IPQueryMiddleware {
            service,
            ip_query: std::sync::Arc::new(self.clone()),
        }))
    }
}

pub struct IPQueryMiddleware<S, T>
where
    T: IPQueryStore,
{
    service: S,
    ip_query: std::sync::Arc<IPQuery<T>>,
}

impl<S, B, T> Service<ServiceRequest> for IPQueryMiddleware<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: IPQueryStore + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = if self.ip_query.forwarded_for {
            req.connection_info()
                .realip_remote_addr()
                .unwrap()
                .to_string()
        } else {
            match req.peer_addr() {
                Some(addr) => addr.ip().to_string(),
                None => {
                    return Box::pin(async {
                        Err(Error::from(actix_web::error::ErrorInternalServerError(
                            "No peer address",
                        )))
                    })
                }
            }
        };

        let fut = self.service.call(req);
        let ip_query_clone = self.ip_query.clone();
        Box::pin(async move {
            let res = fut.await?;
            let ip_info = match ip_query_clone.query_ip(&ip).await {
                Ok(info) => info,
                Err(e) => {
                    return Err(Error::from(actix_web::error::ErrorInternalServerError(
                        e.to_string(),
                    )))
                }
            };
            ip_query_clone.store.store(ip_info)?;
            Ok(res)
        })
    }
}

/// Define the IPQueryStore trait
pub trait IPQueryStore: Send + Sync + Clone {
    fn store(&self, ip_info: IPInfo) -> Result<(), std::io::Error>;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn my_ip() {
        let ip = ipapi::query_own_ip().await.unwrap();
        println!("{:?}", ip);
    }
}
