use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use http::{Request, Response};
use tower::{Layer, Service};
use tracing::info;

/// A Tower layer that logs gRPC requests and responses.
#[derive(Clone, Copy)]
pub struct GrpcLoggingLayer;

impl<S> Layer<S> for GrpcLoggingLayer {
    type Service = GrpcLoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GrpcLoggingService { inner }
    }
}

/// A Tower service that logs gRPC requests and responses.
#[derive(Clone)]
pub struct GrpcLoggingService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for GrpcLoggingService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();

        // Extract gRPC method from URI path (format: /package.Service/Method)
        let path = req.uri().path().to_string();
        let method = path
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();
        let service = path
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        info!(
            grpc.service = %service,
            grpc.method = %method,
            "gRPC request started"
        );

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let result = inner.call(req).await;
            let duration_ms = start.elapsed().as_millis();

            match &result {
                Ok(response) => {
                    let status = response
                        .headers()
                        .get("grpc-status")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("0");
                    info!(
                        grpc.service = %service,
                        grpc.method = %method,
                        grpc.status = %status,
                        duration_ms = %duration_ms,
                        "gRPC request completed"
                    );
                }
                Err(_) => {
                    info!(
                        grpc.service = %service,
                        grpc.method = %method,
                        grpc.status = "error",
                        duration_ms = %duration_ms,
                        "gRPC request failed"
                    );
                }
            }

            result
        })
    }
}
