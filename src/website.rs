use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use warp::Filter;
use warp::http::Response;
use crate::page::{Page, PageType};

pub struct Website {
	landing: Page,
	not_found: Page,
	pages: Arc<HashMap<String, Page>>,
}
impl Website {
	pub fn new(landing: Page, not_found: Page) -> Self {
		Self {
			landing,
			not_found,
			pages: Arc::new(HashMap::new()),
		}
	}

	#[inline]
	pub fn add_page(&mut self, page: Page) -> &mut Self {
		Arc::get_mut(&mut self.pages)
			.expect("Cannot mutate pages after server start")
			.insert(page.path_string(), page);
		self
	}

	pub async fn start(&self, ip: [u8; 4], port: u16) {
		println!(
			"Server running on http://{}.{}.{}.{}:{}",
			ip[0], ip[1], ip[2], ip[3], port
		);

		let pages = self.pages.clone();
		let landing = self.landing.clone();
		let not_found = self.not_found.clone();

		let routes = warp::path::full()
			.and(warp::header::headers_cloned())
			.map(move |path: warp::path::FullPath, headers: warp::http::HeaderMap| {
				let timer = Instant::now();
				let raw = path.as_str();

				let (page, status) = if raw == "/" {
					(&landing, 200)
				} else if let Some(p) = pages.get(raw) {
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

				let resp = builder.body(page.html().clone()).unwrap();

				println!("{} -> {} in {:?}", raw, status, timer.elapsed());
				return resp;
			});

		warp::serve(routes).run((ip, port)).await;
	}
}
