use crate::compression::{gzip_bytes, gzip_html};
use std::net::IpAddr;
use std::sync::Arc;
use warp::http::Response;
use warp::hyper::body::Bytes;

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
				let compressed = gzip_bytes(&bytes);
				let mut res = Response::new(Bytes::from(compressed));
				let headers = res.headers_mut();
				headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
				headers.insert("Content-Encoding", "gzip".parse().unwrap());
				res
			},

			ApiResponse::Html(html_str) => {
				let compressed = gzip_html(&html_str);
				let mut res = Response::new(compressed);
				let headers = res.headers_mut();
				headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());
				headers.insert("Content-Encoding", "gzip".parse().unwrap());
				headers.insert("Cache-Control", "no-store, must-revalidate".parse().unwrap());
				res
			},

			ApiResponse::Redirect(url) => {
				let mut res = Response::new(Bytes::new());
				*res.status_mut() = warp::http::StatusCode::SEE_OTHER; // 303
				res.headers_mut().insert("Location", url.parse().unwrap());
				res
			},

			ApiResponse::LoginSuccess(token, url) => {
				let mut res = Response::new(Bytes::new());
				*res.status_mut() = warp::http::StatusCode::SEE_OTHER;
				let headers = res.headers_mut();
				headers.insert("Location", url.parse().unwrap());
				let cookie = format!("auth_token={}; Path=/; HttpOnly; SameSite=Strict", token);
				headers.insert("Set-Cookie", cookie.parse().unwrap());
				res
			},

			ApiResponse::Unauthorized(msg) => {
				let mut res = Response::new(Bytes::from(msg.into_bytes()));
				*res.status_mut() = warp::http::StatusCode::UNAUTHORIZED;
				res
			},

			ApiResponse::None => {
				let mut res = Response::new(Bytes::new());
				*res.status_mut() = warp::http::StatusCode::SEE_OTHER;
				res
			},

			ApiResponse::SharedBinary(bytes) => {
				let mut res = Response::new(bytes);
				res.headers_mut().insert("Content-Type", "application/octet-stream".parse().unwrap());
				res
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