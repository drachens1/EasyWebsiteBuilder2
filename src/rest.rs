use std::sync::Arc;
use warp::http::Response;

#[derive(Clone)]
pub enum ApiResponse {
	Binary(Vec<u8>),
}
impl ApiResponse {
	pub fn into_response(self) -> Response<Vec<u8>> {
		match self {
			ApiResponse::Binary(bytes) => {
				Response::builder()
					.header("Content-Type", "application/octet-stream")
					.body(bytes)
					.unwrap()
			},
		}
	}
}

pub struct ApiEndpoint {
	pub path: String,
	pub method: Method,
	pub handler: Arc<dyn Fn() -> ApiResponse + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
	GET,
	POST,
	DELETE,
}

impl ApiEndpoint {
	pub fn new(
		path: impl Into<String>,
		method: Method,
		handler: impl Fn() -> ApiResponse + Send + Sync + 'static
	) -> Self {
		Self {
			path: path.into(),
			method,
			handler: Arc::new(handler),
		}
	}

	pub fn binary(
		path: impl Into<String>,
		method: Method,
		handler: impl Fn() -> Vec<u8> + Send + Sync + 'static
	) -> Self {
		Self::new(path, method, move || ApiResponse::Binary(handler()))
	}
}