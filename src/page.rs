use std::sync::Arc;
use crate::compression::gzip_html;

#[derive(Debug)]
pub struct Page {
	path: String,
	html: Arc<Vec<u8>>,
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

	#[inline]
	pub fn content_type(&self) -> &'static str {
		match self.page_type {
			PageType::Html => "text/html; charset=utf-8",
			PageType::Css => "text/css; charset=utf-8",
			PageType::Javascript => "application/javascript; charset=utf-8",
		}
	}
	#[inline] pub fn path(&self) -> &str { &self.path }
	#[inline] pub fn html(&self) -> &Arc<Vec<u8>> { &self.html }
}

#[derive(Debug)]
pub enum PageType {
	Css,
	Javascript,
	Html,
}
