use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use warp::http::Response;

#[derive(Clone)]
pub enum ApiResponse {
	Binary(Vec<u8>),
	Redirect(String),
	Unauthorized(String),
	LoginSuccess(String, String),
	None,
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
			ApiResponse::Redirect(url) => {
				Response::builder()
					.status(303)
					.header("Location", url)
					.body(Vec::new())
					.unwrap()
			},
			ApiResponse::Unauthorized(msg) => {
				Response::builder()
					.status(401)
					.body(msg.into_bytes())
					.unwrap()
			},
			ApiResponse::LoginSuccess(token, url) => {
				Response::builder()
					.status(303)
					.header("Location", url)
					.header("Set-Cookie", format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict", token))
					.body(Vec::new())
					.unwrap()
			},
			ApiResponse::None => {
				Response::builder()
					.status(303)
					.body(Vec::new())
					.unwrap()
			}
		}
	}
}

pub struct ApiRequest {
	pub ip: IpAddr,
	pub body: Vec<u8>,
	pub method: warp::http::Method,
}
impl ApiRequest {
	pub fn ip_string(&self) -> String {
		format!("{}", self.ip)
	}
}

pub struct ApiEndpoint {
	pub path: String,
	pub method: Method,
	pub handler: Arc<dyn Fn(ApiRequest) -> ApiResponse + Send + Sync>,
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
		handler: impl Fn(ApiRequest) -> ApiResponse + Send + Sync + 'static
	) -> Self {
		Self {
			path: path.into(),
			method,
			handler: Arc::new(handler),
		}
	}
}