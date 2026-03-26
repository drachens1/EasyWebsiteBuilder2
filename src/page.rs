use crate::compression::{generate_etag, gzip_html};

#[derive(Debug, Clone)]
pub struct Page {
	path: String,
	html: Vec<u8>,
	page_type: PageType,
	etag: String,
	requires_auth: bool,
	serve_type: ServeType,
}
impl Page {
	pub fn new_html(path: impl Into<String>, html: &str) -> Page {
		let html = gzip_html(html);
		Self {
			path: path.into(),
			etag: generate_etag(&html),
			html,
			page_type: PageType::Html,
			requires_auth: false,
			serve_type: ServeType::Static,
		}
	}

	pub fn new_css(path: impl Into<String>, css: &str) -> Page {
		let css = gzip_html(css);
		Self {
			path: path.into(),
			etag: generate_etag(&css),
			html: css,
			page_type: PageType::Css,
			requires_auth: false,
			serve_type: ServeType::Static,
		}
	}

	pub fn new_js(path: impl Into<String>, js: &str) -> Page {
		let js = gzip_html(js);
		Self {
			path: path.into(),
			etag: generate_etag(&js),
			html: js,
			page_type: PageType::Javascript,
			requires_auth: false,
			serve_type: ServeType::Static,
		}
	}

	pub fn new_webp(path: impl Into<String>, webp_bytes: Vec<u8>) -> Page {
		let etag = generate_etag(&webp_bytes);
		Self {
			path: path.into(),
			html: webp_bytes,
			etag,
			page_type: PageType::Image,
			requires_auth: false,
			serve_type: ServeType::Static,
		}
	}

	pub fn check(mut self) -> Self {
		self.serve_type = ServeType::Check;
		self
	}

	pub fn dynamic(mut self) -> Self {
		self.serve_type = ServeType::Dynamic;
		self
	}

	pub fn cache_for(mut self, i: u32) -> Self {
		self.serve_type = ServeType::Timed(i);
		self
	}

	pub fn immutable(mut self, i: u32) -> Self {
		self.serve_type = ServeType::Immutable(i);
		self
	}

	pub fn serve_type(mut self, serve_type: ServeType) -> Self {
		self.serve_type = serve_type;
		self
	}

	#[inline]
	pub fn content_type(&self) -> &'static str {
		match self.page_type {
			PageType::Html => "text/html; charset=utf-8",
			PageType::Css => "text/css; charset=utf-8",
			PageType::Javascript => "application/javascript; charset=utf-8",
			PageType::Image => "image/webp",
		}
	}

	pub fn private(mut self) -> Self {
		self.requires_auth = true;
		self
	}

	#[inline] pub fn requires_auth(&self) -> bool { self.requires_auth }
	#[inline] pub fn cache_control(&self) -> String {
		match self.serve_type {
			ServeType::Dynamic => "no-store, must-revalidate".to_string(),
			ServeType::Check => "no-cache".to_string(),
			ServeType::Immutable(seconds) => format!("public, max-age={}, immutable", seconds),
			ServeType::Timed(seconds) => format!("public, max-age={}", seconds),
			ServeType::Static => "public, max-age=31536000, immutable".to_string(),
		}
	}
	#[inline] pub fn etag(&self) -> &str { &self.etag }
	#[inline] pub fn page_type(&self) -> &PageType { &self.page_type }
	#[inline] pub fn path(&self) -> &str { &self.path }
	#[inline] pub fn path_string(&self) -> String { self.path.clone() }
	#[inline] pub fn html(&self) -> &Vec<u8> { &self.html }
}

#[derive(Debug, Clone)]
pub enum ServeType {
	Dynamic,
	Check,
	Immutable(u32),
	Timed(u32),
	Static,
}

#[derive(Debug, Clone)]
pub enum PageType {
	Css,
	Javascript,
	Html,
	Image
}
