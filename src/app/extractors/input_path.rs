use axum::extract::{FromRequestParts, Path};
use http::request::Parts;
use serde::de::DeserializeOwned;

use crate::app::api_error::ApiError;

pub struct InputPath<T>(pub T);

impl<T, S> FromRequestParts<S> for InputPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let path = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::invalid_path_param())?;
        Ok(InputPath(path.0))
    }
}
