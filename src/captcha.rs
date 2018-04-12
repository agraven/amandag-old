use futures::{Future, Stream};
use futures::future::ok;

//use gotham::state::State;

use hyper::{self, Client, Method, Request};
use hyper::header::ContentType;
use hyper_tls::HttpsConnector;

use mime::APPLICATION_WWW_FORM_URLENCODED;

use serde_json;

use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;

use std::sync::mpsc::channel;
use std::thread;

use error::{Error, Result};

/// Representation of reCAPTCHA response
#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
	success: bool,
	challenge_ts: String,
	hostname: String,
}

type CaptchaFuture = Future<Item = bool, Error = hyper::Error>;

/// Verifies a reCAPTCHA. Expects input to be URL-encoded, will break otherwise.
/// Note that if the input contains characters that needs encoding, it's
/// probably invalid anyway
pub fn verify(secret: &str, response: &str) -> Result<()> {
	let (sender, receiver) = channel();
	let secret = secret.to_owned();
	let response = response.to_owned();
	thread::spawn(move || {
		let mut core = Core::new().unwrap();
		let handle = &core.handle();
		let result = core.run(run_verify(handle, secret, response))
			.unwrap();
		sender.send(result).unwrap();
	});
	let recv = receiver.recv()?;
	match recv {
		true => Ok(()),
		false => Err(Error::Captcha),
		//Err(e) => Err(e.into()),
	}
}

fn run_verify(
	handle: &Handle,
	secret: String,
	response: String,
) -> Box<CaptchaFuture> {
	let mut request = Request::new(
		Method::Post,
		"https://www.google.com/recaptcha/api/siteverify"
			.parse()
			.unwrap(),
	);
	let query = format!("secret={}&response={}", secret, response);
	request
		.headers_mut()
		.set(ContentType(APPLICATION_WWW_FORM_URLENCODED));
	request.set_body(query);

	let client = Client::configure()
		.connector(HttpsConnector::new(1, &handle).unwrap())
		.build(&handle);

	let f = client
		.request(request)
		.and_then(|res| res.body().concat2())
		.and_then(|body| ok(String::from_utf8(body.to_vec()).unwrap()))
		.and_then(
			|body| ok(serde_json::from_str::<CaptchaResponse>(&body).unwrap()),
		)
		.and_then(|res| ok(res.success));
	Box::new(f)
}
