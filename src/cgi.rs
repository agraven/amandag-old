use std::env;
use std::io;
use std::collections::HashMap;
use std::os::unix::ffi::OsStringExt;

/// Builds a map of key-value pairs from GET request
pub fn get_get() -> Option<HashMap<String, String>> {
	if let Some(get) = env::var_os("QUERY_STRING") {
		let len = get.len();
		Some(split_request_data(get.into_vec(), len))
	} else {
		None
	}
}

/// Builds a map of key-value pairs from POST request
pub fn get_post() -> Option<HashMap<String, String>> {
	let mut post = String::new();
	if let Ok(size) = io::stdin().read_line(&mut post) {
		Some(split_request_data(post.into_bytes(), size))
	} else {
		None
	}
}

/// Converts data from a POST or GET request into a key/value map
fn split_request_data(data: Vec<u8>, length: usize) -> HashMap<String, String> {
	// Parsing index
	let mut i = 0;
	// HashMap containing values
	let mut map = HashMap::new();

	while i < length {
		let mut name = String::new();
		let mut value = String::new();

		// Add character at index to name until '='
		while i < length && data[i] as char != '=' {
			name.push(data[i] as char);
			i += 1;
		}
		i += 1;

		// Add character at index to value until '&' or end of string
		while i < length && data[i] as char != '&' {
			value.push(data[i] as char);
			i += 1;
		}
		map.insert(name, value);
	}
	return map;
}
