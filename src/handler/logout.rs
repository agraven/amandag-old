use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::header::SetCookie;
use mime;

use auth;
use error::{self, Error, Result};

pub fn handle(mut state: State) -> (State, Response) {
	match run(&mut state) {
		Ok(response) => (state, response),
		Err(e) => {
			let content = error::print(e).into_bytes();
			let response = create_response(
				&state,
				StatusCode::Ok,
				Some((content, mime::TEXT_HTML)),
			);
			(state, response)
		}
	}
}

fn run(state: &mut State) -> Result<Response> {
	let session = auth::get_session(&state)?;
	if session.user == "guest" {
		return Err(Error::LoggedOut);
	}
	auth::logout(&session.id)?;

	let content = format!(
		include_str!("../web/index.html"),
		head = "",
		content =
			"<article><h1>Logged out</h1>Successfully logged out</article>",
		userinfo = "",
		title = "Error"
	).into_bytes();
	let mut response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);
	let cookie = String::from("session=0; Max-Age=0");
	response
		.headers_mut()
		.set::<SetCookie>(SetCookie(vec![cookie]));
	Ok(response)
}
