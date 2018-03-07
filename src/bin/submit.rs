extern crate amandag;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;

use amandag::auth;
use amandag::captcha;
use amandag::cgi;
use amandag::mysql;

// Error definitions
error_chain! {
	foreign_links {
		Captcha(captcha::Error);
		Io(std::io::Error);
		Sql(mysql::Error);
		Utf8(std::string::FromUtf8Error);
	}
	links {
		Auth(auth::Error, auth::ErrorKind);
		Cookie(cgi::Error, cgi::ErrorKind);
	}
	errors {
		Param(s: &'static str) {
			description("missing POST parameter"),
			display("Missing POST value: {}", s),
		}
		Unauthorized {
			description("User does not have permission to submit articles"),
			display("You do not have permission to submit articles"),
		}
	}
}
use ErrorKind::Param;

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Error",
			head = "",
			user = "",
			content = format!(
				"<article><h1>Internal server error</h1>\
				 The page could not be displayed because of an internal \
				 error: {}</article>",
				e
			),
		);
	}
}

fn run() -> Result<()> {
	let password = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let session = auth::get_session()?;
	// If article was submitted, don't print submisstion form
	if cgi::request_method_is("POST") {
		// Get a map of POST values
		let map = cgi::get_post().expect("Failed to get post values");

		// Get values from POST
		let get = |key: &'static str| -> Result<&String> {
			map.get(key).ok_or(Param(key).into())
		};
		let title = get("title")?;
		let content = get("content")?;
		let category = get("category")?;
		let response = get("g-recaptcha-response")?;
		let secret = String::from_utf8(
			File::open("secret/submit-captcha")?
				.bytes()
				.map(|b| b.unwrap())
				.collect(),
		)?;

		// Verify captcha
		captcha::verify(&secret, &response)?;

		// Check user permissions
		if session.user != "amanda" {
			return Err(ErrorKind::Unauthorized.into());
		}

		// Insert article into database
		let url = format!(
			"mysql://submit:{}@localhost:3306/amandag",
			password
		);
		mysql::Pool::new(url)?.prep_exec(
			"INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
			(title, content, category),
		)?;

		println!(
			include_str!("../web/index.html"),
			title = "Article submitted",
			head = "",
			user = session.user,
			content = "<article>Article submitted.</article>"
		);

		return Ok(());
	}
	// Print submission form
	println!("{}\n", include_str!("../web/http-headers"));
	println!(
		include_str!("../web/index.html"),
		title = "Amanda Graven's homepage - Submit article",
		head = "",
		user = session.user,
		content = include_str!("../web/submit.html")
	);

	Ok(())
}
