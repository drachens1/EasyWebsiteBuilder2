use crate::page::{Page, PageType};
use crate::rest::{ApiEndpoint, Method};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use warp::http::Response;
use warp::Filter;

pub struct Website {
	landing: Page,
	not_found: Page,
	pages: Arc<HashMap<String, Page>>,
	endpoints: Arc<HashMap<String, ApiEndpoint>>,
}
impl Website {
	pub fn new(landing: Page, not_found: Page) -> Self {
		Self {
			landing,
			not_found,
			pages: Arc::new(HashMap::new()),
			endpoints: Arc::new(HashMap::new()),
		}
	}

	#[inline]
	pub fn add_page(&mut self, page: Page) -> &mut Self {
		Arc::get_mut(&mut self.pages)
			.expect("Cannot mutate pages after server start")
			.insert(page.path_string(), page);
		self
	}

	#[inline]
	pub fn add_rest_endpoint(&mut self, endpoint: ApiEndpoint) -> &mut Self {
		Arc::get_mut(&mut self.endpoints)
			.expect("Cannot mutate endpoints after server start")
			.insert(endpoint.path.clone(), endpoint);
		self
	}

	pub async fn start(&self, ip: [u8; 4], port: u16) {
		println!("Server running on http://{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port);

		let pages = self.pages.clone();
		let endpoints = self.endpoints.clone();
		let landing = self.landing.clone();
		let not_found = self.not_found.clone();

		let routes = warp::path::full()
			.and(warp::method())
			.and(warp::header::headers_cloned())
			.map(move |full_path: warp::path::FullPath, method: warp::http::Method, headers: warp::http::HeaderMap| {
				let timer = Instant::now();
				let raw_path = full_path.as_str();

				if let Some(endpoint) = endpoints.get(raw_path) {
					let method_matches = match endpoint.method {
						Method::GET => method == warp::http::Method::GET,
						Method::POST => method == warp::http::Method::POST,
						Method::DELETE => method == warp::http::Method::DELETE,
					};

					if method_matches {
						let resp = (endpoint.handler)().into_response();
						println!("API: {} -> {} in {:?}", raw_path, resp.status(), timer.elapsed());
						return resp;
					}
				}

				let (page, status) = if raw_path == "/" {
					(&landing, 200)
				} else if let Some(p) = pages.get(raw_path) {
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
