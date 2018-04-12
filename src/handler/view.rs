use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Response, StatusCode};

use mime;

use article::Article;
use auth;
use cgi;
use comment::{Comment, CommentList};
use error::{self, Error, Result};
use mysql;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct PathExtractor {
	id: i32,
}

const SELECT_COMMENT_COUNT: &str = "SELECT COUNT(*) AS comment_count \
                                    FROM comments WHERE post_id = ?";
const SELECT_ARTICLE: &'static str =
	"SELECT id, title, content, post_time, edit_time, category \
	 FROM posts WHERE id = ?";

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
	// Establish connection to MySQL server
	let pool =
		mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
	let session = auth::get_session(&state)?;
	// Get article from database
	let article = {
		let row = pool.first_exec(SELECT_ARTICLE, (id,))?
			.ok_or(Error::InvalidId(id as u64))?;
		// Bind values from row
		let (id, title, content, post_time, edit_time, category) =
			mysql::from_row(row);
		// Get amount of comments
		let comment_count = mysql::from_row(
			pool.first_exec(SELECT_COMMENT_COUNT, (id,))?
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
	};

	let comments: Vec<Comment> = pool.prep_exec(
		"SELECT id, author, user, content, post_time, parent_id \
		 FROM comments WHERE post_id = ?",
		(id,),
	).map(|result| {
		result
			.map(|x| x.unwrap())
			.map(|row| {
				let (id, author, user, content, post_time, parent_id) =
					mysql::from_row(row);
				Comment {
					id,
					author,
					user,
					content,
					post_time,
					parent_id,
				}
			})
			.collect()
	})?;

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
