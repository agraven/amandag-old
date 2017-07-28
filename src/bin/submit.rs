extern crate amandag;
extern crate mysql;
extern crate time;

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;

use amandag::Post;
use amandag::strings;
use amandag::cgi;

fn main() {
	if env::var_os("REQUEST_METHOD") == Some(OsString::from("POST")) {
		let post_map = cgi::get_post().unwrap_or(HashMap::new());

		// Make sure we have all necessary POST values
		if !(post_map.contains_key("title") && post_map.contains_key("content") && post_map.contains_key("user") && post_map.contains_key("password")) {
			
		}

		let pool = mysql::Pool::new(format!("mysql://{}:{}@localhost:3306/amandag"));
	}
}
