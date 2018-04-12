use Article;
use auth;
use cgi;
use error::{self, Result};
use mime;
use mysql;

use gotham::http::response::create_response;
use gotham::state::State;

use hyper::{Response, StatusCode};

const SELECT_COMMENT_COUNT: &'static str = "SELECT COUNT(*) AS comment_count \
                                            FROM comments WHERE post_id = ?";
const SELECT_ARTICLES: &'static str =
	"SELECT id, title, content, post_time, edit_time, category \
	 FROM posts \
	 ORDER BY post_time DESC \
	 LIMIT 20";

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
	let pool =
		mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
	let session = auth::get_session(&state)?;
	// Select posts from SQL DATABASE
	let selected: Vec<Article> = pool.prep_exec(SELECT_ARTICLES, ())
		.map(|result| {
			// Iterate through rows
			result
				.map(|x| x.unwrap())
				.map(|row| {
					let (id, title, content, post_time, edit_time, category) =
						mysql::from_row(row);
					// Get amount of comments on post
					let comment_count = mysql::from_row(
						pool.first_exec(SELECT_COMMENT_COUNT, (id,))
							.unwrap()
							.unwrap(),
					);
					Article {
						id,
						title,
						content,
						post_time,
						edit_time,
						category,
						comment_count,
					}
				})
				.collect()
		})
		.unwrap();

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
