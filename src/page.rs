use crate::compression::{generate_etag, gzip_html};
use warp::http::Response;
use warp::hyper::body::Bytes;

#[derive(Debug, Clone)]
pub struct Page {
	path: String,
	data: Bytes,
	page_type: PageType,
	etag: String,
	requires_auth: bool,
	serve_type: ServeType,
	response: Response<Bytes>,
}
impl Page {
	pub fn new_html(path: impl Into<String>, html: &str) -> Page {
		let html = gzip_html(html);
		let mut s = Self {
			path: path.into(),
			etag: generate_etag(&html),
			data: html,
			page_type: PageType::Html,
			requires_auth: false,
			serve_type: ServeType::Static,
			response: Default::default()
		};
		s.update_response();
		s
	}

	pub fn new_css(path: impl Into<String>, css: &str) -> Page {
		let css = gzip_html(css);
		let mut s = Self {
			path: path.into(),
			etag: generate_etag(&css),
			data: css,
			page_type: PageType::Css,
			requires_auth: false,
			serve_type: ServeType::Static,
			response: Default::default(),
		};
		s.update_response();
		s
	}

	pub fn new_js(path: impl Into<String>, js: &str) -> Page {
		let js = gzip_html(js);
		let mut s = Self {
			path: path.into(),
			etag: generate_etag(&js),
			data: js,
			page_type: PageType::Javascript,
			requires_auth: false,
			serve_type: ServeType::Static,
			response: Default::default(),
		};
		s.update_response();
		s
	}

	pub fn new_webp(path: impl Into<String>, webp_bytes: Vec<u8>) -> Page {
		let etag = generate_etag(&webp_bytes);
		let mut s = Self {
			path: path.into(),
			data: Bytes::from_owner(webp_bytes),
			etag,
			page_type: PageType::Image,
			requires_auth: false,
			serve_type: ServeType::Static,
			response: Default::default(),
		};
		s.update_response();
		s
	}

	fn update_response(&mut self) {
		let mut builder = Response::builder()
			.status(200)
			.header("Content-Type", self.page_type.response_str())
			.header("Vary", "Accept-Encoding")
			.header("Cache-Control", self.serve_type.response_str())
			.header("ETag", self.etag.clone());

		if !matches!(self.page_type, PageType::Image) {
			builder = builder.header("Content-Encoding", "gzip");
		}

		self.response = builder.body(self.data.clone()).unwrap();
	}

	pub fn check(mut self) -> Self {
		self.serve_type = ServeType::Check;
		self.update_response();
		self
	}

	pub fn dynamic(mut self) -> Self {
		self.serve_type = ServeType::Dynamic;
		self.update_response();
		self
	}

	pub fn cache_for(mut self, i: u32) -> Self {
		self.serve_type = ServeType::Timed(i);
		self.update_response();
		self
	}

	pub fn immutable(mut self, i: u32) -> Self {
		self.serve_type = ServeType::Immutable(i);
		self.update_response();
		self
	}

	pub fn serve_type(mut self, serve_type: ServeType) -> Self {
		self.serve_type = serve_type;
		self.update_response();
		self
	}
	pub fn private(mut self) -> Self {
		self.requires_auth = true;
		self.update_response();
		self
	}

	#[inline] pub fn requires_auth(&self) -> bool { self.requires_auth }
	#[inline] pub fn response(&self) -> Response<Bytes> {
		self.response.clone()
	}
	#[inline] pub fn etag(&self) -> &str { &self.etag }
	#[inline] pub fn page_type(&self) -> &PageType { &self.page_type }
	#[inline] pub fn path(&self) -> &str { &self.path }
	#[inline] pub fn path_string(&self) -> String { self.path.clone() }
	#[inline] pub fn data(&self) -> &Bytes { &self.data }
}

#[derive(Debug, Clone)]
pub enum ServeType {
	Dynamic,
	Check,
	Immutable(u32),
	Timed(u32),
	Static,
}
impl ServeType {
	pub fn response_str(&self) -> String {
		match self {
			ServeType::Dynamic => "no-store, must-revalidate".to_string(),
			ServeType::Check => "no-cache".to_string(),
			ServeType::Immutable(seconds) => format!("public, max-age={}, immutable", seconds),
			ServeType::Timed(seconds) => format!("public, max-age={}", seconds),
			ServeType::Static => "public, max-age=31536000, immutable".to_string(),
		}
	}
}

#[derive(Debug, Clone)]
pub enum PageType {
	Css,
	Javascript,
	Html,
	Image
}
impl PageType {
	#[inline] pub fn response_str(&self) -> &'static str {
		match self {
			PageType::Html => "text/html; charset=utf-8",
			PageType::Css => "text/css; charset=utf-8",
			PageType::Javascript => "application/javascript; charset=utf-8",
			PageType::Image => "image/webp",
		}
	}
}
