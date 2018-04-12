use hoedown::{Html, Markdown, Render};
use hoedown::renderer::html;
use url::form_urlencoded;

use std::collections::HashMap;

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

pub fn print_user_info(user: &str) -> String {
	if user == "guest" {
		include_str!("web/guest-info.html").to_owned()
	} else {
		format!(include_str!("web/user-info.html"), user = user)
	}
}

/// Converts data from a POST or GET request into a key/value map
pub fn request_decode(data: Vec<u8>) -> HashMap<String, String> {
	form_urlencoded::parse(data.as_slice())
		.into_owned()
		.collect::<HashMap<String, String>>()
}
