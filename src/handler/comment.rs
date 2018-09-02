use futures::{future, Future, Stream};

use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Body, Response, StatusCode};

use mime;

use std::fs::File;
use std::io::Read;

use auth;
use captcha;
use cgi;
use comment::Comment;
use db;
use error::{Error, Result};
use time;


pub fn handle(mut state: State) -> Box<HandlerFuture> {
	let f = Body::take_from(&mut state)
		.concat2()
		.then(|result| match result {
			Ok(body) => match run(&mut state, body.to_vec()) {
				Ok(response) => future::ok((state, response)),
				Err(e) => {
					let response = create_response(
						&state,
						StatusCode::InternalServerError,
						Some((format!("{}", e).into_bytes(), mime::TEXT_PLAIN)),
					);
					future::ok((state, response))
				}
			},
			Err(e) => return future::err((state, e.into_handler_error())),
		});
	Box::new(f)
}

fn run(state: &State, post: Vec<u8>) -> Result<Response> {
	let session = auth::get_session(&state).unwrap();
	let map = cgi::request_decode(post);

	let get = |key: &'static str| -> Result<&String> {
		map.get(key).ok_or(Error::MissingParam(key))
	};

	// Verify captcha if user is guest
	if session.user == "guest" {
		let secret = String::from_utf8(
			File::open("secret/comment-captcha")?
				.bytes()
				.map(|b| b.unwrap())
				.collect(),
		)?;
		let response = get("g-recaptcha-response").unwrap();
		captcha::verify(&secret, &response)?;
	}

	// Get values
	let author = get("name")?;
	let content = get("content")?;
	let post_id = get("id")?.parse::<i64>()?;
	let parent_id = get("parent")?.parse::<i64>()?;

	let id = db::insert_comment(
        &session.user,
        &author,
        &content,
        post_id,
        parent_id,
	)?;

	let post_time = time::get_time();

	let content = format!(
		"{}",
		Comment {
			id,
			author: author.to_owned(),
			user: session.user.clone(),
			content: content.to_owned(),
			post_time,
			parent_id,
		}.display(&session.user == "guest")
	).into_bytes();
	let response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);
	Ok(response)
}
