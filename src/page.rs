use std::fs;
use crate::compression::gzip_html;

#[derive(Debug, Clone)]
pub struct Page {
	path: String,
	html: Vec<u8>,
	total_views: u64,
	current_viewers: u32,
	page_type: PageType,
}
impl Page {
	pub fn new_from_html_str(path: impl Into<String>, html: &str, total_views: u64) -> Page {
		Self {
			path: path.into(),
			html: gzip_html(html),
			total_views,
			current_viewers: 0,
			page_type: PageType::Html,
		}
	}

	pub fn new_css(path: impl Into<String>, css: &str) -> Page {
		Self {
			path: path.into(),
			html: gzip_html(css),
			total_views: 0,
			current_viewers: 0,
			page_type: PageType::Css,
		}
	}

	pub fn new_js(path: impl Into<String>, js: &str) -> Page {
		Self {
			path: path.into(),
			html: gzip_html(js),
			total_views: 0,
			current_viewers: 0,
			page_type: PageType::Javascript,
		}
	}

	pub fn new_png(path: impl Into<String>, png_bytes: Vec<u8>) -> Page {
		Self {
			path: path.into(),
			html: png_bytes,
			total_views: 0,
			current_viewers: 0,
			page_type: PageType::Image,
		}
	}

	pub fn new_png_from_file(path: impl Into<String>, file_path: impl Into<String>) -> Page {
		let bytes = fs::read(file_path.into())
			.expect("Failed to read PNG file");
		Self::new_png(path, bytes)
	}

	#[inline]
	pub fn content_type(&self) -> &'static str {
		match self.page_type {
			PageType::Html => "text/html; charset=utf-8",
			PageType::Css => "text/css; charset=utf-8",
			PageType::Javascript => "application/javascript; charset=utf-8",
			PageType::Image => "image/png",
		}
	}
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
