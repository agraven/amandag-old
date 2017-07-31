use std::env;
use std::io;
use std::collections::HashMap;
use std::os::unix::ffi::OsStringExt;

/// Builds a map of key-value pairs from GET request
pub fn get_get() -> Option<HashMap<String, String>> {
	if let Some(get_str) = env::var_os("QUERY_STRING") {
		Some(split_request_data(get_str.into_vec()))
	} else {
		None
	}
}

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
	let mut post = String::new();
	if let Ok(_) = io::stdin().read_line(&mut post) {
		Some(split_request_data(post.into_bytes()))
	} else {
		None
	}
}

trait Deescape {
	fn deescape_http(&self) -> String;
}

impl Deescape for String {
	fn deescape_http(&self) -> String {
		self.replace('+', " ")
		.replace("%20", " ")
		.replace("%21", "!")
		.replace("%22", "\"")
		.replace("%23", "#")
		.replace("%24", "$")
		.replace("%25", "%")
		.replace("%26", "&")
		.replace("%27", "'")
		.replace("%28", "(")
		.replace("%29", ")")
		.replace("%2A", "*")
		.replace("%2B", "+")
		.replace("%2C", ",")
		.replace("%2D", "-")
		.replace("%2E", ".")
		.replace("%2F", "/")
		.replace("%3A", ":")
		.replace("%3B", ";")
		.replace("%3C", "<")
		.replace("%3D", "=")
		.replace("%3E", ">")
		.replace("%3F", "?")
		.replace("%5B", "[")
		.replace("%5C", "\\")
		.replace("%5D", "]")
		.replace("%5E", "^")
		.replace("%5F", "^")
		.replace("%60", "`")
		.replace("%7B", "{")
		.replace("%7C", "|")
		.replace("%7D", "}")
		.replace("%7E", "~")
	}
}

/// Converts data from a POST or GET request into a key/value map
fn split_request_data(data: Vec<u8>) -> HashMap<String, String> {
	// HashMap containing key-value pairs
	let mut map = HashMap::new();

	for pair in data.split(|val| *val == '&' as u8) {
		// Get a Vec of name and value slices by splitting at '='
		let fields: Vec<&[u8]> = pair.splitn(2, |val| *val == '=' as u8)
			.map(|slice| slice.clone()).collect();
		match fields.len() {
			2 => {
				map.insert(
					String::from_utf8_lossy(fields[0]).into_owned(),
					String::from_utf8_lossy(fields[1]).into_owned().deescape_http()
				);
			}
			1 => {
				map.insert(
					String::from_utf8_lossy(fields[0]).into_owned(),
					String::new()
				);
			}
			_ => (),
		}
	}
	return map;
}
