use std::sync::Arc;
use std::time::Instant;
use warp::Filter;
use warp::http::Response;
use crate::page::Page;

//Page 0 is landing, Page 1 is 404, Page 2 + dynamic
pub struct Website {
	pages: Arc<Vec<Page>>,
}
impl Website {
	pub fn new() -> Self {
		Self { pages: Arc::new(Vec::new()) }
	}

	#[inline]
	pub fn add_page(&mut self, page: Page) -> &mut Self {
		Arc::get_mut(&mut self.pages).unwrap().push(page);
		self
	}

	#[inline]
	pub fn set_landing_page(&mut self, page: Page) {
		let pages = Arc::get_mut(&mut self.pages).unwrap();
		if pages.len() > 0 {
			pages[0] = page;
		}else {
			pages.push(page);
		}
	}

	#[inline]
	pub fn set_404(&mut self, page: Page) {
		let pages = Arc::get_mut(&mut self.pages).unwrap();
		if pages.len() > 1 {
			pages[1] = page;
		}else {
			pages.push(page);
		}
	}

	pub async fn start(&self, ip: [u8; 4], port: u16) {
		println!("Server running on http://{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port);

		let pages = self.pages.clone();
		let dynamic_route = warp::any()
			.map(move || pages.clone())
			.and(warp::path::full())
			.map(|pages: Arc<Vec<Page>>, path: warp::path::FullPath| {
				let path_str = path.as_str();
				
				if path_str == "/" {
					if let Some(landing) = pages.get(0) {
						let timer = Instant::now();
						let response = Response::builder()
							.header("Content-Encoding", "gzip")
							.header("Content-Type", "text/html; charset=utf-8")
							.body(landing.html().as_ref().clone())
							.unwrap();
						println!("landing page took {:?}", timer.elapsed());
						return response;
					}
				}

				for page in pages.iter().skip(2) {
					let page_path = page.path();
					if path_str == format!("/{}", page_path) || path_str == page_path {
						return Response::builder()
							.header("Content-Encoding", "gzip")
							.header("Content-Type", page.content_type())
							.body(page.html().as_ref().clone())
							.unwrap();
					}
				}

				let body = if let Some(not_found) = pages.get(1) {
					not_found.html().as_ref().clone()
				} else {
					panic!("404 Page not defined")
				};

				Response::builder()
					.status(404)
					.header("Content-Encoding", "gzip")
					.header("Content-Type", "text/html; charset=utf-8")
					.body(body)
					.unwrap()
			});

		let routes = dynamic_route.map(|resp| Box::new(resp) as Box<dyn warp::Reply>).boxed();

		warp::serve(routes).run((ip, port)).await;
	}
}
