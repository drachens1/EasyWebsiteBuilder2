use crate::compression::{generate_etag, gzip_html};

#[derive(Debug, Clone)]
pub struct Page {
	path: String,
	html: Vec<u8>,
	page_type: PageType,
	etag: String,
	requires_auth: bool,
	dynamic: bool,
}
impl Page {
	pub fn new_from_html_str(path: impl Into<String>, html: &str) -> Page {
		let html = gzip_html(html);
		Self {
			path: path.into(),
			etag: generate_etag(&html),
			html,
			page_type: PageType::Html,
			requires_auth: false,
			dynamic: false,
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
			dynamic: false,
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
			dynamic: false,
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
			dynamic: false,
		}
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

	pub fn dynamic(mut self) -> Self {
		self.dynamic = true;
		self
	}

	#[inline] pub fn requires_auth(&self) -> bool { self.requires_auth }
	#[inline] pub fn cache_control(&self) -> &'static str { if self.dynamic { "no-cache" } else { "public, max-age=31536000, immutable" } }
	#[inline] pub fn etag(&self) -> &str { &self.etag }
	#[inline] pub fn page_type(&self) -> &PageType { &self.page_type }
	#[inline] pub fn path(&self) -> &str { &self.path }
	#[inline] pub fn path_string(&self) -> String { self.path.clone() }
	#[inline] pub fn html(&self) -> &Vec<u8> { &self.html }
}

#[derive(Debug, Clone)]
pub enum PageType {
	Css,
	Javascript,
	Html,
	Image
}
