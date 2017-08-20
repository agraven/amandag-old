extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

use std::string;
use std::fmt::{self, Display, Formatter};

use self::futures::{Future, Stream};
use self::hyper::{Client, Method, Request};
use self::hyper::header::ContentType;
use self::hyper_tls::HttpsConnector;
use self::mime::APPLICATION_WWW_FORM_URLENCODED;
use self::tokio_core::reactor::Core;

// Representation of reCAPTCHA response
#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
	success: bool,
	challenge_ts: String,
	hostname: String,
}

pub enum Error {
	CaptchaError,
	HyperError(hyper::Error),
	IoError(::std::io::Error),
	JsonError(serde_json::Error),
	Utf8Error(string::FromUtf8Error),
	UriError(hyper::error::UriError),
}
use self::Error::*;

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
	(HyperError, hyper::Error),
	(IoError, ::std::io::Error),
	(JsonError, serde_json::Error),
	(Utf8Error, string::FromUtf8Error),
	(UriError, hyper::error::UriError)
];

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(
			f,
			"{}",
			match *self {
				CaptchaError => format!("reCAPTCHA failed"),
				HyperError(ref e) => format!("HTTP error: {}", e),
				IoError(ref e) => format!("I/O error: {}", e),
				JsonError(ref e) => format!("JSON parsing error: {}", e),
				Utf8Error(ref e) => format!("Invalid UTF-8: {}", e),
				UriError(ref e) => format!("URI error: {}", e),
			}
		)?;
		Ok(())
	}
}

pub fn verify(secret: &str, response: &str) -> Result<(), Error> {
	// Initialize hyper/tokio
	let mut core = Core::new()?;
	let client = Client::configure()
		.connector(HttpsConnector::new(1, &core.handle()).unwrap())
		.build(&core.handle());
	// Define request
	let mut request = Request::new(
		Method::Post,
		"https://www.google.com/recaptcha/api/siteverify".parse()?
	);
	// TODO: urlencode query
	let query = format!("secret={}&response={}", secret, response);
	request.headers_mut().set(ContentType(APPLICATION_WWW_FORM_URLENCODED));
	request.set_body(query);
	let work = client.request(request).and_then(|res| {
		res.body().collect()
	});
	// Fetch response
	let body = String::from_utf8(core.run(work)?.iter().fold(
			// Collect body contents by appending chunk contents to accumulator
			Vec::new(),
			|mut acc, c| { acc.extend_from_slice(c); acc }
	))?;
	// Deserialize response
	let response: CaptchaResponse = serde_json::from_str(&body)?;
	if !response.success { return Err(CaptchaError) };
	Ok(())
}
