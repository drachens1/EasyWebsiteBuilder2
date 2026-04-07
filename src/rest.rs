use std::net::IpAddr;
use std::sync::Arc;
use warp::http::Response;
use warp::hyper::body::Bytes;
use crate::compression::{gzip_bytes, gzip_html};

#[derive(Clone)]
pub enum ApiResponse {
	Binary(Vec<u8>),
	SharedBinary(Bytes),
	Html(String),
	Redirect(String),
	Unauthorized(String),
	LoginSuccess(String, String),
	None,
}
impl ApiResponse {
	pub fn into_response(self) -> Response<Bytes> {
		match self {
			ApiResponse::Binary(bytes) => {
				let compressed_bytes = gzip_bytes(&bytes);

				Response::builder()
					.header("Content-Type", "application/octet-stream")
					.header("Content-Encoding", "gzip")
					.body(Bytes::from_owner(compressed_bytes))
					.unwrap()
			},
			ApiResponse::SharedBinary(bytes) => {
				Response::builder()
					.header("Content-Type", "application/octet-stream")
					.header("Access-Control-Allow-Origin", "*")
					.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
					.header("Access-Control-Allow-Headers", "Content-Type, Cookie")
					.body(bytes)
					.unwrap()
			},
			ApiResponse::Html(html_str) => {
				let compressed_html = gzip_html(&html_str);

				Response::builder()
					.status(200)
					.header("Content-Type", "text/html; charset=utf-8")
					.header("Content-Encoding", "gzip")
					.header("Cache-Control", "no-store, must-revalidate")
					.body(compressed_html)
					.unwrap()
			},
			ApiResponse::Redirect(url) => {
				Response::builder()
					.status(303)
					.header("Location", url)
					.body(Bytes::new())
					.unwrap()
			},
			ApiResponse::Unauthorized(msg) => {
				Response::builder()
					.status(401)
					.body(Bytes::from_owner(msg.into_bytes()))
					.unwrap()
			},
			ApiResponse::LoginSuccess(token, url) => {
				Response::builder()
					.status(303)
					.header("Location", url)
					.header("Set-Cookie", format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict", token))
					.body(Bytes::new())
					.unwrap()
			},
			ApiResponse::None => {
				Response::builder()
					.status(303)
					.body(Bytes::new())
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