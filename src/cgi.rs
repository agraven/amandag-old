use hoedown::{Html, Markdown, Render};
use hoedown::renderer::html;
use url::form_urlencoded;

use std::collections::HashMap;
use std::env;
use std::io;
use std::io::Read;
use std::os::unix::ffi::OsStringExt;

error_chain! {
	errors {
		CookiesUndefined {
			description("Cookie environment variable unset"),
			display("Cookies are not defined"),
		}
		OsString {
			description("OsString contains non-valid unicode"),
			display("Failed to process foreign string"),
		}
	}
}

pub trait Encode {
	/// Replace html characters with their escape codes
	fn encode_html(&self) -> String;
	/// Parse markdown and return its HTML form
	fn render_markdown(&self) -> String;
	/// Render markdown and ensure HTML characters are escaped
	fn render_html(&self) -> String;
}

impl Encode for String {
	fn encode_html(&self) -> String {
		self.replace('&', "&amp;")
			.replace('"', "&quot;")
			.replace('<', "&lt;")
			.replace('>', "&gt;")
	}
	fn render_markdown(&self) -> String {
		let markdown = Markdown::new(&self.encode_html());
		let mut html = Html::new(html::Flags::empty(), 0);
		html.render(&markdown)
			.to_str()
			.unwrap()
			.to_string()
	}
	fn render_html(&self) -> String { self.render_markdown() }
}

/// Returns a map of set cookies by parsing the HTTP_COOKIE environment
/// variable. The function is not protected from special characters
pub fn get_cookies() -> Result<HashMap<String, String>> {
	let cookies_raw = env::var_os("HTTP_COOKIE")
        .unwrap_or(::std::ffi::OsString::new());
    let cookies = cookies_raw.to_string_lossy().to_owned();
	let mut map = HashMap::new();
	for pair in cookies.split("; ") {
		let mut iter = pair.splitn(2, '=');
		let key = iter.next().unwrap();
		let value = iter.next().unwrap_or("");
		map.insert(String::from(key), String::from(value));
	}
	Ok(map)
}

/// Builds a map of key-value pairs from GET request
pub fn get_get() -> Option<HashMap<String, String>> {
	if let Some(get_str) = env::var_os("QUERY_STRING") {
		Some(request_decode(get_str.into_vec()))
	} else {
		None
	}
}

/// Checks if the current request method matches the argument
pub fn request_method_is(method: &str) -> bool {
	if let Some(val) = env::var_os("REQUEST_METHOD") {
		return method == val.to_string_lossy();
	}
	false
}

/// Gets a value from a GET request
pub fn get_get_member(name: String) -> Option<String> {
	if let Some(get_map) = get_get() {
		if let Some(value) = get_map.get(name.as_str()) {
			Some(value.clone())
		} else {
			None
		}
	} else {
		None
	}
}

/// Builds a map of key-value pairs from POST request
pub fn get_post() -> Option<HashMap<String, String>> {
	let mut post = Vec::new();
	if let Ok(_) = io::stdin().read_to_end(&mut post) {
		Some(request_decode(post))
	} else {
		None
	}
}

/// Converts data from a POST or GET request into a key/value map
fn request_decode(data: Vec<u8>) -> HashMap<String, String> {
	form_urlencoded::parse(data.as_slice())
		.into_owned()
		.collect::<HashMap<String, String>>()
}
