use futures::{future, Future, Stream};

use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Body, Response, StatusCode};
use hyper::header::SetCookie;

use mime;

use auth;
use cgi;
use error::{self, Error, Result};

pub fn get(state: State) -> (State, Response) {
	match run_get(&state) {
		Ok(content) => {
			let response = create_response(
				&state,
				StatusCode::Ok,
				Some((content, mime::TEXT_HTML)),
			);
			(state, response)
		}
		Err(e) => {
			let content = error::print(e).into_bytes();
			let response = create_response(
				&state,
				StatusCode::InternalServerError,
				Some((content, mime::TEXT_HTML)),
			);
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

fn run_get(state: &State) -> Result<Vec<u8>> {
	let session = auth::get_session(&state)?;
	if session.user != "guest" {
		Ok(format!(
			include_str!("../web/index.html"),
			title = "Login successful",
			head = "",
			userinfo = cgi::print_user_info(&session.user),
			content = "<article><h1>Already logged in</h1> \
			           To sign in as another user, you have to log out first.</article>",
		).into_bytes())
	} else {
		Ok(format!(
			include_str!("../web/index.html"),
			title = "Login",
			content = include_str!("../web/login.html"),
			userinfo = cgi::print_user_info(&session.user),
			head = ""
		).into_bytes())
	}
}
fn run_post(state: &State, post: Vec<u8>) -> Result<Response> {
	let post = cgi::request_decode(post);
	let get = |key: &'static str| -> Result<&String> {
		post.get(key).ok_or(Error::MissingParam(key))
	};
	let user = get("user")?;
	let pass = get("password")?;

	let session = auth::login(&user, &pass)?;

	let content = format!(
        include_str!("../web/index.html"),
        title = "Login successful",
        head = "",
        userinfo = cgi::print_user_info(&user),
        content = format!(
            "<article><h1>Login successful</h1>Successfully logged in as {}</article>",
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
