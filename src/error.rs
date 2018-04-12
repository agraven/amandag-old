use gotham::http::response::create_response;
use gotham::state::State;
use hyper;
use hyper::{Response, StatusCode};
use mime;
use mysql;
use native_tls;
use serde_json;

use std;
use std::error::Error as StdError;
use std::fmt;

/*error_def! Error {
	Captcha => "reCAPTCHA failed" ("reCAPTCHA verification failed"),
	ExpiredToken => "token has expired" ("Token has expired"),
	InvalidId { id: u64 } => "invalid article id" ("Invalid article id: {}", id),
	InvalidToken => "token is invalid" ("Invalid login token"),
	MissingParam { s: &'static str } => "missing POST parameter" ("Missing POST value: {}", s),
	NoSuchToken => "no such token" ("Attemped to login with nonexistant token"),
	NoSuchUser { user: String } => "no such user" ("The user {} doesn't exist", user),
	WrongPassword => "wrong password" ("Wrong password"),
}*/

#[derive(Debug)]
pub enum Error {
	// Foreign errors
	Hyper(hyper::Error),
	Io(std::io::Error),
	Json(serde_json::Error),
	ParseInt(std::num::ParseIntError),
	Recv(std::sync::mpsc::RecvError),
	SqlConversion(mysql::FromRowError),
	Sql(mysql::Error),
	Tls(native_tls::Error),
	Url(hyper::error::UriError),
	Utf8(std::string::FromUtf8Error),

	// Native errors
	Captcha,
	ExpiredToken,
	InvalidId(u64),
	LoggedOut,
	MissingParam(&'static str),
	NoSuchToken,
	NoSuchUser,
	Reserved,
	Unauthorized,
	WrongPassword,
}

impl Error {
	pub fn into_response(self, state: &State) -> Response {
		create_response(
			&state,
			StatusCode::InternalServerError,
			Some((print(self).into_bytes(), mime::TEXT_HTML)),
		)
	}
}

// TODO: actually implement these
impl std::error::Error for Error {
	fn description(&self) -> &str {
		use self::Error::*;
		match self {
			&Hyper(_) => "hyper error",
			&Io(_) => "I/O error",
			&Json(_) => "JSON parsing error",
			&ParseInt(_) => "int parsing error",
			&SqlConversion(_) => "SQL data conversion error",
			&Sql(_) => "SQL error",
			&Tls(_) => "SSL/TLS error",
			&Url(_) => "URL parsing error",
			&Utf8(_) => "invalid UTF-8",

			&Captcha => "reCAPTCHA failed",
			&ExpiredToken => "login token has expired",
			&InvalidId(_) => "Invalid article id",
			&LoggedOut => "Already logged out",
			&MissingParam(_) => "Missing parameter",
			&NoSuchToken => "Login token doesn't exist",
			&NoSuchUser => "User doesn't exist",
			&Reserved => "Username already taken",
			&WrongPassword => "Wrong password",
			_ => "",
		}
	}
	fn cause(&self) -> Option<&StdError> {
		use self::Error::*;
		match self {
			&Hyper(ref e) => Some(e as &StdError),
			&Json(ref e) => Some(e as &StdError),
			&ParseInt(ref e) => Some(e as &StdError),
			&SqlConversion(ref e) => Some(e as &StdError),
			&Sql(ref e) => Some(e as &StdError),
			&Tls(ref e) => Some(e as &StdError),
			&Url(ref e) => Some(e as &StdError),
			&Utf8(ref e) => Some(e as &StdError),

			_ => None,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(cause) = self.cause() {
			return write!(f, "{}: {}", self.description(), cause);
		}
		write!(f, "{}", self.description())
	}
}

impl From<hyper::Error> for Error {
	fn from(e: hyper::Error) -> Error { Error::Hyper(e) }
}
impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Error { Error::Io(e) }
}
impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Error { Error::Json(e) }
}
impl From<std::num::ParseIntError> for Error {
	fn from(e: std::num::ParseIntError) -> Error { Error::ParseInt(e) }
}
impl From<mysql::FromRowError> for Error {
	fn from(e: mysql::FromRowError) -> Error { Error::SqlConversion(e) }
}
impl From<mysql::Error> for Error {
	fn from(e: mysql::Error) -> Error { Error::Sql(e) }
}
impl From<native_tls::Error> for Error {
	fn from(e: native_tls::Error) -> Error { Error::Tls(e) }
}
impl From<hyper::error::UriError> for Error {
	fn from(e: hyper::error::UriError) -> Error { Error::Url(e) }
}
impl From<std::sync::mpsc::RecvError> for Error {
	fn from(e: std::sync::mpsc::RecvError) -> Error { Error::Recv(e) }
}
impl From<std::string::FromUtf8Error> for Error {
	fn from(e: std::string::FromUtf8Error) -> Error { Error::Utf8(e) }
}

pub type Result<T> = std::result::Result<T, Error>;
trait AssertSendSync: Send + Sync + 'static {}
impl AssertSendSync for Error {}
impl<T> AssertSendSync for Result<T>
where
	T: AssertSendSync,
{
}

pub fn print(error: Error) -> String {
	format!(
		include_str!("web/index.html"),
		title = "Internal server error",
		head = "",
		userinfo = "",
		content = format!(
			"<article><h1>Error</h1>{}</article>",
			error.to_string()
		),
	)
}
