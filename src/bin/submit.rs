extern crate amandag;
extern crate mysql;
extern crate time;

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;

//use amandag::Post;
use amandag::strings;
use amandag::cgi;

pub const SUBMIT_CONTENT: &'static str = r##"
		<article>
			<h1>Submit post</h1>
			<form action="submit.cgi" method="post">
				Title:<br>
				<input type="text" name="title">
				<br>Category:<br>
				<input type="text" name="category">
				<p>Content:<br>
				<textarea rows="1" cols="1" name="content"></textarea>

				<p>User:<br>
				<input type="text" name="user">
				<br>Password:<br>
				<input type="password" name="password">

				<p>
				<input type="submit" value="Submit"/>
				<div class="g-recaptcha" data-sitekey="6LdO2SoUAAAAAPOph0HIJ7mUUEnDsG_mfS0AHL1L"></div>
			</form>
		</article>"##;

pub const SUBMIT_ERROR: &'static str = r##"\t\t<article>
			<h1>Server error</h1>
			<p>Article submission failed: missing post values.
		</article>"##;

pub fn format_document_header_with_captcha(title: &str) -> String {
	format!(r##"Content-type: text/html; charset=utf-8
X-Powered-By: Rust/1.16.0
Content-Language: en

<!DOCTYPE html>
<html>
<head>
	<title>{}</title>
	<meta name="author" content="Amanda Graven">
	<meta name="description" content="Personal homepage of Amanda Graven">

	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="stylesheet" type="text/css" href="/style.css">
	<script src="https://www.google.com/recaptcha/api.js"></script>
</head>
<body>
	<div id="headbar">
		<div id="headbar-content">
			<div class="title" style="float: left;">Amanda's terrible homepage</div>
			<ul class="navbar">
				<li><a href="/">Home</a></li>
				<li><a href="/about">About</a></li>
			</ul>
		</div>
	</div>
	<div id="easter-egg"> </div>
	<main>
		<div id="body-title">
			<div class="title">Amanda's terrible homepage</div>
		</div>"##, title)
}

fn main() {
	// If article was submitted, don't print submisstion form
	if env::var_os("REQUEST_METHOD") == Some(OsString::from("POST")) {
		// Get a map of POST values
		let post_map = cgi::get_post().unwrap_or(HashMap::new());
		// Make sure we have all necessary POST values
		let mut has_keys = true;
		for key in &["title", "content", "category", "user", "password"] {
			if !post_map.contains_key(&key.to_string()) { has_keys = false; }
		}
		if !has_keys {
			println!("{}", strings::format_document_header("Error"));
			println!("{}", SUBMIT_ERROR);
			println!("{}", strings::DOCUMENT_FOOTER);
			return;
		}
		// Get values from POST
		let user = post_map.get("user").unwrap();
		let password = post_map.get("password").unwrap();
		let title = post_map.get("title").unwrap();
		let content = post_map.get("content").unwrap();
		let category = post_map.get("category").unwrap();
		// Insert article into database
		if let Ok(pool) = mysql::Pool::new(format!("mysql://{}:{}@localhost:3306/amandag", user, password)) {
			pool.prep_exec("INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
				(title, content, category)).unwrap();

			println!("{}", strings::format_document_header("Article submitted"));
			println!("<article>Article submitted, you can view it. Here's a preview of its contents:
			<h1>{}</h1><p>{}</article>", title, content);
			println!("{}", strings::DOCUMENT_FOOTER);
		} else {
			println!("{}", strings::format_document_header("Error"));
			println!("\t\t<article>Failed to submit article: error connecting to database</article>");
			println!("{}", strings::DOCUMENT_FOOTER);
		}

		return;
	}
	// Print submission form
	println!("{}", format_document_header_with_captcha("Submit post"));
	println!("{}", SUBMIT_CONTENT);
	println!("{}", strings::DOCUMENT_FOOTER);
}
