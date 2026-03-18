use crate::page::{Page, PageType};
use crate::rest::{ApiEndpoint, ApiRequest, Method};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use warp::http::Response;
use warp::Filter;

pub struct Website {
	landing: Page,
	not_found: Page,
	pages: Arc<RwLock<HashMap<String, Page>>>,
	endpoints: Arc<RwLock<HashMap<String, ApiEndpoint>>>,
	secret: String,
}
impl Website {
	pub fn new(secret: String, landing: Page, not_found: Page) -> Self {
		Self {
			landing,
			not_found,
			pages: Arc::new(RwLock::new(HashMap::new())),
			endpoints: Arc::new(RwLock::new(HashMap::new())),
			secret,
		}
	}

	#[inline]
	pub async fn add_page(&self, page: Page) {
		let mut lock = self.pages.write().unwrap();
		lock.insert(page.path_string(), page);
	}

	#[inline]
	pub async fn remove_page(&self, path_string: &str) -> Option<Page> {
		let mut lock = self.pages.write().unwrap();
		lock.remove(path_string)
	}

	#[inline]
	pub async fn add_rest_endpoint(&self, endpoint: ApiEndpoint) {
		let mut lock = self.endpoints.write().unwrap();
		lock.insert(endpoint.path.clone(), endpoint);
	}

	pub async fn start(&self, ip: [u8; 4], port: u16) {
		println!("Server running on http://{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port);

		let pages = self.pages.clone();
		let endpoints = self.endpoints.clone();
		let landing = self.landing.clone();
		let not_found = self.not_found.clone();

		let auth_token = format!("auth_token={}", self.secret);

		let routes = warp::path::full()
			.and(warp::method())
			.and(warp::header::headers_cloned())
			.and(warp::body::bytes())
			.map(move |full_path: warp::path::FullPath, method: warp::http::Method, headers: warp::http::HeaderMap, body: warp::hyper::body::Bytes| {
				let timer = Instant::now();
				let raw_path = full_path.as_str();

				let is_authenticated = headers.get("cookie")
					.and_then(|c| c.to_str().ok())
					.map(|c| c.contains(&auth_token))
					.unwrap_or(false);

				let endpoints_lock = endpoints.read().unwrap();

				if let Some(endpoint) = endpoints_lock.get(raw_path) {
					let method_matches = match endpoint.method {
						Method::GET => method == warp::http::Method::GET,
						Method::POST => method == warp::http::Method::POST,
						Method::DELETE => method == warp::http::Method::DELETE,
					};

					if method_matches {
						let req = ApiRequest {
							body: body.to_vec(),
							method: method.clone(),
						};

						let resp = (endpoint.handler)(req).into_response();
						println!("API: {} -> {} in {:?}", raw_path, resp.status(), timer.elapsed());
						return resp;
					}
				}
				drop(endpoints_lock);

				let pages_lock = pages.read().unwrap();
				let (page, status) = if raw_path == "/" {
					(&landing, 200)
				} else if let Some(p) = pages_lock.get(raw_path) {
					if p.requires_auth() && !is_authenticated {
						println!("AUTH: Access Denied for {} -> Redirecting to /", raw_path);
						return Response::builder()
							.status(303)
							.header("Location", "/")
							.body(Vec::new())
							.unwrap();
					}
					(p, 200)
				} else {
					(&not_found, 404)
				};

				if let Some(inm) = headers.get("if-none-match") {
					if inm.to_str().ok() == Some(page.etag()) {
						return Response::builder()
							.status(304)
							.header("ETag", page.etag())
							.header("Cache-Control", page.cache_control())
							.body(Vec::new())
							.unwrap();
					}
				}

				let mut builder = Response::builder()
					.status(status)
					.header("Content-Type", page.content_type())
					.header("Vary", "Accept-Encoding")
					.header("Cache-Control", page.cache_control())
					.header("ETag", page.etag());

				if !matches!(page.page_type(), PageType::Image) {
					builder = builder.header("Content-Encoding", "gzip");
				}

				println!("STATIC: {} -> {} in {:?}", raw_path, status, timer.elapsed());
				builder.body(page.html().clone()).unwrap()
			});

		warp::serve(routes).run((ip, port)).await;
	}
}
