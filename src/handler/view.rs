use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Response, StatusCode};

use mime;

use auth;
use cgi;
use comment::CommentList;
use db;
use error::{self, Result};

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct PathExtractor {
	id: i32,
}

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
	let id = PathExtractor::borrow_from(&state).id;
	let session = auth::get_session(&state)?;

	let article = db::select_article(id as u64)?;
	let comments = db::select_comments(id as u64)?;

	let content = format!(
		include_str!("../web/index.html"),
		title = article.title,
		head = format!(
			include_str!("../web/view-head.html"),
			id = article.id,
			description = if article.content.len() > 200 {
				article.content[..200].to_owned() + "…"
			} else {
				article.content.clone()
			},
		),
		userinfo = cgi::print_user_info(&session.user),
		content = format!(
			"{}{}{}",
			article.display(),
			include_str!("../web/comment-form.html"),
			comments.display(session.user == "guest")
		),
	).into_bytes();
	Ok(content)
}
