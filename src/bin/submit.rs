extern crate amandag;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate mysql;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::{self, Read, Write};

use self::futures::{Future, Stream};
use self::hyper::{Client, Method, Request};
use self::hyper::header::ContentType;
use self::hyper_tls::HttpsConnector;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use self::tokio_core::reactor::Core;

use amandag::strings;
use amandag::cgi;

// Representation of reCAPTCHA response
#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
	success: bool,
	challenge_ts: String,
	hostname: String,
}

// Error definitions
enum Error {
	CaptchaError,
	HyperError(hyper::Error),
	IoError(io::Error),
	JsonError(serde_json::Error),
	MissingError(&'static str),
	SqlError(mysql::error::Error),
	UriError(hyper::error::UriError),
	Utf8Error(std::string::FromUtf8Error),
}
use Error::*;

// Macro for quick implementation of From<T> for Error
macro_rules! impl_error {
	( $( ($l:ident, $f:ty) ),* ) => {
		$(
			impl From<$f> for Error {
				fn from(err: $f) -> Error {
					Error::$l(err)
				}
			}
		)*
	}
}
impl_error!(
	(HyperError, hyper::Error),
	(IoError, io::Error),
	(JsonError, serde_json::Error),
	(MissingError, &'static str),
	(SqlError, mysql::Error),
	(UriError, hyper::error::UriError),
	(Utf8Error, std::string::FromUtf8Error)
);

fn main() {
	match run() {
		Ok(()) => (),
		Err(e) => {
			println!("{}{}{}{}",
				strings::format_document_header("Error"),
				"<article><h1>Internal server error</h1>The page could \
				not be displayed because of an internal error: ",
				match e {
					CaptchaError => format!("reCAPTCHA failed"),
					HyperError(e) => format!("HTTP error: {}", e),
					IoError(e) => format!("I/O error: {}", e),
					JsonError(e) => format!("JSON parsing error: {}", e),
					MissingError(s) => format!("Missing parameter '{}'", s),
					SqlError(e) => format!("Database error: {}", e),
					UriError(e) => format!("URI parsing error: {}", e),
					Utf8Error(err) => format!(
							"Illegal UTF-8 at {}",
							err.utf8_error().valid_up_to()
						),
				},
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

		write!(File::create("debug.log")?, "{:?}", map)?;

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

		// Verify captcha
		// Start by fetching response from verification server
		let mut core = Core::new()?;
		let client = Client::configure()
			.connector(HttpsConnector::new(1, &core.handle()).unwrap())
			.build(&core.handle());
		let mut request = Request::new(
			Method::Post,
			"https://www.google.com/recaptcha/api/siteverify".parse()?
		);
		let query = format!(
			"secret={secret}&response={response}",
			secret = String::from_utf8(
				File::open("secret/submit-captcha")?
					.bytes()
					.map(|b| b.unwrap())
					.collect()
			)?,
			response = response
		);
		request.headers_mut().set(ContentType(APPLICATION_WWW_FORM_URLENCODED));
		request.set_body(query);
		let work = client.request(request).and_then(|res| {
			res.body().collect()
		});
		let body = String::from_utf8(core.run(work)?.iter().fold(
				Vec::new(),
				|mut acc, ref c| { acc.extend_from_slice(&c); acc }
		))?;
		// Deserialize response
		let response: CaptchaResponse = serde_json::from_str(&body)?;
		if !response.success { return Err(CaptchaError) };

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
