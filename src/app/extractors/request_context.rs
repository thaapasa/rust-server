use axum::Extension;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::api_error::ApiError;
use crate::context::{ContextImpl, Environment};

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Extension::<Environment>::from_request_parts(parts, state).await {
            Ok(env) => Ok(RequestContext(
                ContextImpl::new(env.0).await.map_err(ApiError::from)?,
            )),
            Err(e) => panic!("Environment missing: {e}"),
        }
    }
}

pub struct RequestContext(pub ContextImpl);
