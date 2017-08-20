extern crate amandag;
extern crate mysql;
extern crate time;

use std::fs::File;
use std::io::{self, Read};
use std::fmt::{self, Display, Formatter};

use amandag::captcha;
use amandag::cgi;
use amandag::strings;

// Error definitions
enum Error {
	CaptchaError(captcha::Error),
	IoError(io::Error),
	MissingError(&'static str),
	SqlError(mysql::error::Error),
	Utf8Error(std::string::FromUtf8Error),
}
use Error::*;

// Macro for quick implementation of From<T> for Error
macro_rules! impl_error {
	[ $( ($l:ident, $f:ty) ),* ] => {
		$(
			impl From<$f> for Error {
				fn from(err: $f) -> Error {
					Error::$l(err)
				}
			}
		)*
	}
}
impl_error![
	(CaptchaError, captcha::Error),
	(IoError, io::Error),
	(SqlError, mysql::Error),
	(Utf8Error, std::string::FromUtf8Error)
];

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", match *self {
			CaptchaError(ref e) => format!("reCAPTCHA failed: {}", e),
			IoError(ref e) => format!("I/O error: {}", e),
			MissingError(ref s) => format!("Missing parameter '{}'", s),
			SqlError(ref e) => format!("Database error: {}", e),
			Utf8Error(ref e) => format!("Invalid UTF-8: {}", e),
		})?;
		Ok(())	
	}
}

fn main() {
	match run() {
		Ok(()) => (),
		Err(err) => {
			println!("{}{}{}{}",
				strings::format_document_header("Error"),
				"<article><h1>Internal server error</h1>The page could \
				not be displayed because of an internal error: ",
				err,
				format!("</article>\n{}", strings::DOCUMENT_FOOTER)
			);
		},
	};
}

fn run() -> Result<(), Error> {
	// If article was submitted, don't print submisstion form
	if cgi::request_method_is("POST") {
		// Get a map of POST values
		let map = cgi::get_post().expect("Failed to get post values");

		// Get values from POST
		let get = |key: &'static str| -> Result<&String, Error> {
			map.get(key).ok_or(MissingError(key))
		};
		let user = get("user")?;
		let password = get("password")?;
		let title = get("title")?;
		let content = get("content")?;
		let category = get("category")?;
		let response = get("g-recaptcha-response")?;
		let secret = String::from_utf8(
			File::open("secret/submit-captcha")?
				.bytes()
				.map(|b| b.unwrap())
				.collect()
		)?;

		// Verify captcha
		captcha::verify(&secret, &response)?;

		// Insert article into database
		let url = format!("mysql://{}:{}@localhost:3306/amandag", user, password);
		mysql::Pool::new(url)?
			.prep_exec(
				"INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
				(title, content, category)
			)?;

		println!("{}", strings::format_document_header("Article submitted"));
		println!("<article>Article submitted. Here's a preview of its contents:
			<h1>{}</h1><p>{}</article>", title, content);
		println!("{}", strings::DOCUMENT_FOOTER);

		return Ok(());
	}
	// Print submission form
	println!("{}", strings::format_captcha_header("Submit post"));
	println!("{}", strings::SUBMIT_CONTENT);
	println!("{}", strings::DOCUMENT_FOOTER);

	Ok(())
}
