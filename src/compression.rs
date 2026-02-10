use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

#[inline]
pub fn gzip_html(content: &str) -> Vec<u8> {
	gzip_html_bytes(content.as_bytes())
}

#[inline]
pub fn gzip_html_bytes(content: &[u8]) -> Vec<u8> {
	let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
	encoder.write_all(content).unwrap();
	encoder.finish().unwrap()
}

