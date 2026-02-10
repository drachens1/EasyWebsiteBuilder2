use std::sync::Arc;
use crate::compression::gzip_html;

pub struct Page {
	path: String,
	html: Arc<Vec<u8>>,
	total_views: u64,
	current_viewers: u32,
}
impl Page {
	pub fn new_from_html_str(path: impl Into<String>, html: &str, total_views: u64) -> Page {
		Self {
			path: path.into(),
			html: gzip_html(html),
			total_views,
			current_viewers: 0,
		}
	}

	pub fn new_from_html_string(path: impl Into<String>, html: String, total_views: u64) -> Page {
		Self {
			path: path.into(),
			html: gzip_html(&html),
			total_views,
			current_viewers: 0,
		}
	}

	#[inline] pub fn path(&self) -> &str { &self.path }
	#[inline] pub fn html(&self) -> &Arc<Vec<u8>> { &self.html }
}

