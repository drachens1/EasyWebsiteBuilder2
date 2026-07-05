use crate::page::Page;
use crate::rest::{ApiEndpoint, ApiRequest, Method};
use hashbrown::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use warp::http::Response;
use warp::hyper::body::Bytes;
use warp::Filter;
use DrachLogger::Logger;

pub struct Website {
	landing: Page,
	not_found: Page,
	pages: Arc<RwLock<HashMap<String, Page>>>,
	endpoints: Arc<RwLock<HashMap<String, ApiEndpoint>>>,
	logger: Arc<Logger>,
	secret: Option<String>,
	response_304: Response<Bytes>,
	response_auth_denied: Response<Bytes>,
	fetch_ip: bool,
}
impl Website {
	pub fn new(log_path: &str, landing: Page, not_found: Page) -> Self {
		Self {
			landing,
			not_found,
			pages: Arc::new(RwLock::new(HashMap::new())),
			endpoints: Arc::new(RwLock::new(HashMap::new())),
			secret: None,
			response_304: Response::builder()
				.status(304)
				.body(Bytes::new())
				.unwrap(),
			response_auth_denied: Response::builder()
				.status(303)
				.header("Location", "/")
				.body(Bytes::new())
				.unwrap(),
			fetch_ip: false,
			logger: Arc::new(Logger::new(log_path, true)),
		}
	}

	pub fn new_secret(log_path: &str, secret: impl Into<String>, landing: Page, not_found: Page) -> Self {
		Self {
			landing,
			not_found,
			pages: Arc::new(RwLock::new(HashMap::new())),
			endpoints: Arc::new(RwLock::new(HashMap::new())),
			secret: Some(secret.into()),
			response_304: Response::builder()
				.status(304)
				.header("Cache-Control", "no-store, must-revalidate")
				.body(Bytes::new())
				.unwrap(),
			response_auth_denied: Response::builder()
				.status(303)
				.header("Location", "/")
				.body(Bytes::new())
				.unwrap(),
			fetch_ip: false,
			logger: Arc::new(Logger::new(log_path, true)),
		}
	}

	#[inline] pub fn fetch_ip(mut self) -> Self {
		self.fetch_ip = true;
		self
	}

	#[inline]
	pub async fn add_page(&self, page: Page) {
		let mut lock = self.pages.write().unwrap();
		lock.insert(page.path_string(), page);
	}

	#[inline]
	pub async fn remove_page(&self, path_string: &str) -> Option<Page> {
		let mut lock = self.pages.write().unwrap();
		lock.remove(path_string)
	}

	#[inline]
	pub async fn add_rest_endpoint(&self, endpoint: ApiEndpoint) {
		let mut lock = self.endpoints.write().unwrap();
		lock.insert(endpoint.path.clone(), endpoint);
	}

	pub async fn start(&self, ip: [u8; 4], port: u16) {
		self.logger.try_info(&format!("Server running on http://{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port));

		let website_arc = Arc::new(self.clone_refs());

		let base = warp::path::full()
			.and(warp::method())
			.and(warp::header::headers_cloned())
			.and(warp::body::bytes())
			.and(warp::query::<HashMap<String, String>>());

		let routes = if self.fetch_ip {
			base.and(remote_addr())
				.map(move |path, method, headers, body, queries, addr| {
					website_arc.handle_request(path, method, headers, body, queries, addr)
				}).boxed()
		} else {
			base.map(move |path, method, headers, body, queries| {
				website_arc.handle_request(path, method, headers, body, queries, None)
			}).boxed()
		};

		warp::serve(routes).run((ip, port)).await;
	}

	fn handle_request(
		&self,
		full_path: warp::path::FullPath,
		method: warp::http::Method,
		headers: warp::http::HeaderMap,
		body: Bytes,
		queries: HashMap<String, String>,
		addr: Option<SocketAddr>,
	) -> Response<Bytes> {
		#[cfg(debug_assertions)]
		let timer = Instant::now();
		let raw_path = full_path.as_str();

		let endpoints_lock = self.endpoints.read().unwrap();
		if let Some(endpoint) = endpoints_lock.get(raw_path) {
			let method_matches = match endpoint.method {
				Method::GET => method == warp::http::Method::GET,
				Method::POST => method == warp::http::Method::POST,
				Method::DELETE => method == warp::http::Method::DELETE,
			};

			if method_matches {
				let client_ip = addr.map(|s| s.ip())
					.unwrap_or_else(|| IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

				let req = ApiRequest {
					ip: client_ip,
					body: body.to_vec(),
					method: method.clone(),
					queries,
				};

				let resp = (endpoint.handler)(req).into_response();
				#[cfg(debug_assertions)]
				self.logger.try_debug(&format!("API: {} -> {} in {:?}", raw_path, resp.status(), timer.elapsed()));
				return resp;
			}
		}
		drop(endpoints_lock);

		let pages_lock = self.pages.read().unwrap();
		let auth_token = self.secret.as_ref().map(|s| format!("auth_token={}", s));

		let (page, status) = if raw_path == "/" {
			(&self.landing, 200)
		} else if let Some(p) = pages_lock.get(raw_path) {
			if p.requires_auth() {
				let is_authenticated = headers.get("cookie")
					.and_then(|c| c.to_str().ok())
					.map(|c| c.contains(&auth_token.unwrap()))
					.unwrap_or(false);

				if !is_authenticated {
					self.logger.try_debug(&format!("AUTH: Access Denied for {}", raw_path));
					return self.response_auth_denied.clone();
				}
			}
			(p, 200)
		} else {
			(&self.not_found, 404)
		};

		if let Some(inm) = headers.get("if-none-match") {
			if inm.to_str().ok() == Some(page.etag()) {
				return self.response_304.clone();
			}
		}


		#[cfg(debug_assertions)]
		self.logger.try_debug(&format!("STATIC: {} -> {} in {:?}", raw_path, status, timer.elapsed()));
		#[cfg(not(debug_assertions))]
		self.logger.try_info(&format!("STATIC: {} -> {}", raw_path, status));
		page.response()
	}

	fn clone_refs(&self) -> Self {
		Self {
			landing: self.landing.clone(),
			not_found: self.not_found.clone(),
			pages: self.pages.clone(),
			endpoints: self.endpoints.clone(),
			logger: self.logger.clone(),
			secret: self.secret.clone(),
			response_304: self.response_304.clone(),
			response_auth_denied: self.response_auth_denied.clone(),
			fetch_ip: self.fetch_ip,
		}
	}
}

pub fn remote_addr() -> impl Filter<Extract = (Option<SocketAddr>,), Error = Infallible> + Copy {
	warp::header::optional::<String>("x-forwarded-for")
		.and(warp::filters::ext::optional::<SocketAddr>())
		.map(|forwarded: Option<String>, remote: Option<SocketAddr>| {
			if let Some(ip) = forwarded
				.and_then(|s| s.split(',').next()?.trim().parse::<IpAddr>().ok())
			{
				return Some(SocketAddr::new(ip, 0));
			}
			remote
		})
		.recover(|_| async {
			Ok::<Option<SocketAddr>, Infallible>(None)
		})
		.unify()
}