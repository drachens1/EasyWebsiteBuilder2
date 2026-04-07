use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::io::Write;
use warp::hyper::body::Bytes;

#[inline]
pub fn gzip_html(content: &str) -> Bytes {
	Bytes::from_owner(gzip_bytes(minify_html(content).as_bytes()))
}

#[inline]
pub fn minify_html(html: &str) -> String {
	html.replace("\n", "").replace("\t", "").replace("  ", " ")
}

#[inline]
pub fn gzip_bytes(content: &[u8]) -> Vec<u8> {
	let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
	encoder.write_all(content).unwrap();
	encoder.finish().unwrap()
}

#[inline]
pub fn generate_etag(bytes: &[u8]) -> String {
	let mut hasher = Sha256::new();
	hasher.update(bytes);
	let hash = hasher.finalize();
	format!("\"{}\"", hex::encode(hash))
}

