// src/proto.rs
//
// Axum extractor and responder for application/x-protobuf content.
// Handlers that accept `Protobuf<T>` decode the request body with prost.
// Handlers that return `Protobuf<T>` encode with prost and set the correct
// Content-Type header.
//

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use prost::Message;

pub mod blogs {
    include!(concat!(env!("OUT_DIR"), "/blogs.rs"));
}

pub struct Protobuf<T>(pub T);

impl<T, S> FromRequest<S> for Protobuf<T>
where
    T: Message + Default,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        T::decode(bytes)
            .map(Protobuf)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
    }
}

impl<T: Message> IntoResponse for Protobuf<T> {
    fn into_response(self) -> Response {
        let bytes = self.0.encode_to_vec();
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/x-protobuf"),
            )],
            bytes,
        )
            .into_response()
    }
}
