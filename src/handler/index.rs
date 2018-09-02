use auth;
use cgi;
use db;
use error::{self, Result};
use mime;

use gotham::http::response::create_response;
use gotham::state::State;

use hyper::{Response, StatusCode};

pub fn handle(state: State) -> (State, Response) {
	match run(&state) {
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

fn run(state: &State) -> Result<Vec<u8>> {
	let session = auth::get_session(&state)?;
	// Select posts from SQL DATABASE
	let selected = db::select_articles()?;

	// Print document
	let mut articles = String::new();
	for post in selected {
		articles.push_str(&post.display());
	}
	let content = format!(
		include_str!("../web/index.html"),
		userinfo = cgi::print_user_info(&session.user),
		title = "Amanda Graven's homepage",
		head = "",
		content = articles,
	).into_bytes();
	Ok(content)
}
