extern crate hoedown;
extern crate url;

use self::url::form_urlencoded;
use self::hoedown::{Markdown, Html, Render};
use self::hoedown::renderer::html;

use std::env;
use std::io;
use std::io::Read;
use std::collections::HashMap;
use std::os::unix::ffi::OsStringExt;

pub trait Encode {
	/// Replace html characters with their escape codes
	fn encode_html(&self) -> String;
	fn render_markdown(&self) -> String;
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
		html.render(&markdown).to_str().unwrap().to_string()
	}
	fn render_html(&self) -> String {
		self.encode_html().render_markdown()
	}
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
	form_urlencoded::parse(data.as_slice()).into_owned().collect::<HashMap<String, String>>()
}
