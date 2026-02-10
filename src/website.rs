use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use warp::Filter;
use warp::http::Response;
use crate::page::Page;

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
		println!("Server running on http://{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port);

		let pages = self.pages.clone();
		let landing = self.landing.clone();
		let not_found = self.not_found.clone();

		let routes = warp::path::full()
			.map(move |path: warp::path::FullPath| {
			let timer = Instant::now();
			let raw = path.as_str();
				let status;

			let page = if raw == "/" {
				status = 200;
				&landing
			} else {
				if let Some(page) = pages.get(raw) {
					status = 200;
					page
				} else {
					status = 404;
					&not_found
				}
			};
			let resp = Response::builder()
				.status(status)
				.header("Content-Encoding", "gzip")
				.header("Content-Type", page.content_type())
				.body(page.html().clone())
				.unwrap();

			println!("{} -> {:?} in {:?}", raw, status, timer.elapsed());
			resp
		});

		warp::serve(routes).run((ip, port)).await;
	}
}
