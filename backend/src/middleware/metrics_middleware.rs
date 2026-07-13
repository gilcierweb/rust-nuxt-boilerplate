use std::{rc::Rc, time::Instant};

use actix_web::{
    Error,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::AppState;

pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let metrics = req
            .app_data::<actix_web::web::Data<AppState>>()
            .map(|state| state.metrics.clone());
        let started_at = Instant::now();

        Box::pin(async move {
            let result = service.call(req).await;
            let elapsed = started_at.elapsed();

            if let Some(registry) = metrics {
                let status = result
                    .as_ref()
                    .map(|response| response.status().as_u16())
                    .unwrap_or(500);
                registry.record(&method, &path, status, elapsed);
            }

            result
        })
    }
}
