use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::api_error::ApiError;
use crate::context::{Environment, RootContext};

pub struct RequestContext(pub RootContext);

impl<S: Send + Sync> FromRequestParts<S> for RequestContext {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let env = parts
            .extensions
            .get::<Environment>()
            .expect("Environment missing");
        let ctx = RootContext::new(env.clone());
        Ok(RequestContext(ctx))
    }
}
