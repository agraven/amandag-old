use std;

use futures::{Future, Stream};
use hyper;
use hyper::{Client, Method, Request};
use hyper::header::ContentType;
use hyper_tls::HttpsConnector;
use mime::APPLICATION_WWW_FORM_URLENCODED;
use native_tls;
use serde_json;
use tokio_core::reactor::Core;

/// Representation of reCAPTCHA response
#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
	success: bool,
	challenge_ts: String,
	hostname: String,
}

error_chain! {
	foreign_links {
		Hyper(hyper::Error);
		Io(std::io::Error);
		Json(serde_json::Error);
		Tls(native_tls::Error);
		Utf8(std::string::FromUtf8Error);
		Uri(hyper::error::UriError);
	}
	errors {
		Captcha {
			description("reCAPTCHA failed"),
			display("reCAPTCHA verification failed"),
		}
	}
}

/// Verifies a reCAPTCHA. Expects input to be URL-encoded, will break otherwise.
/// Note that if the input contains characters that needs encoding, it's
/// probably invalid anyway
pub fn verify(secret: &str, response: &str) -> Result<()> {
	// Initialize hyper/tokio
	let mut core = Core::new()?;
	let client = Client::configure()
		.connector(HttpsConnector::new(1, &core.handle())?)
		.build(&core.handle());
	// Define request
	let mut request = Request::new(
		Method::Post,
		"https://www.google.com/recaptcha/api/siteverify".parse()?
	);
	let query = format!("secret={}&response={}", secret, response);
	request.headers_mut().set(ContentType(APPLICATION_WWW_FORM_URLENCODED));
	request.set_body(query);
	let work = client.request(request).and_then(|res| {
		res.body().concat2()
	});
	// Fetch response
	let body = String::from_utf8(core.run(work)?.to_vec())?;
	// Deserialize response
	let response: CaptchaResponse = serde_json::from_str(&body)?;
	if !response.success {
		Err(ErrorKind::Captcha.into())
	} else {
		Ok(())
	}
}
