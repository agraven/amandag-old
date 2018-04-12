use futures::{future, Future, Stream};

use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Body, Response, StatusCode};
use hyper::header::SetCookie;

use std::fs::File;
use std::io::Read;

use mime;

use auth;
use captcha;
use cgi;
use error::{self, Error, Result};

const RESERVED: &[&str] = &["guest", "admin"];

pub fn get(state: State) -> (State, Response) {
	match run_get(&state) {
		Ok(response) => (state, response),
		Err(e) => {
			let response = e.into_response(&state);
			(state, response)
		}
	}
}

pub fn post(mut state: State) -> Box<HandlerFuture> {
	let f = Body::take_from(&mut state)
		.concat2()
		.then(|result| match result {
			Ok(body) => match run_post(&state, body.to_vec()) {
				Ok(response) => future::ok((state, response)),
				Err(e) => {
					let content = error::print(e).into_bytes();
					let response = create_response(
						&state,
						StatusCode::InternalServerError,
						Some((content, mime::TEXT_HTML)),
					);
					future::ok((state, response))
				}
			},
			Err(e) => return future::err((state, e.into_handler_error())),
		});

	Box::new(f)
}

fn run_get(state: &State) -> Result<Response> {
	let session = auth::get_session(&state)?;

	let content = format!(
		include_str!("../web/index.html"),
		title = "Sign up",
		content = include_str!("../web/signup.html"),
		head =
			"<script src='https://www.google.com/recaptcha/api.js'></script>",
		userinfo = cgi::print_user_info(&session.user),
	).into_bytes();
	let response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);
	Ok(response)
}

fn run_post(state: &State, post: Vec<u8>) -> Result<Response> {
	let post = cgi::request_decode(post);
	let get = |key: &'static str| -> Result<&String> {
		post.get(key).ok_or(Error::MissingParam(key))
	};
	let user = get("user")?;
	// Check if username is reserved
	if RESERVED.contains(&user.as_str()) {
		return Err(Error::Reserved);
	}
	// Check reCAPTCHA
	let response = get("g-recaptcha-response")?;
	let secret = String::from_utf8(
		File::open("secret/comment-captcha")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;

	captcha::verify(&secret, &response)?;

	let pass = get("password")?;
	let name = get("name")?;

	auth::create(&user, &pass, &name)?;

	let session = auth::login(&user, &pass)?;

	let content = format!(
		include_str!("../web/index.html"),
		title = "Signup successful",
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = format!(
			"<article><h1>Signup successful</h1>Successfully created user {}",
			user
		)
	).into_bytes();
	let mut response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);
	{
		let headers = response.headers_mut();
		let cookie = format!("token={}; SameSite=Strict", session.id);
		headers.set::<SetCookie>(SetCookie(vec![cookie]));
	}
	Ok(response)
}
