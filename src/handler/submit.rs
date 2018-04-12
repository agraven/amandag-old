use futures::{future, Future, Stream};

use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Body, Response, StatusCode};

use mime;

use std::fs::File;
use std::io::Read;

use article::Article;
use auth;
use cgi;
use cgi::Encode;
use error::{self, Error, Result};
use mysql;
use time;

#[derive(Deserialize, StateData, StaticResponseExtender, Clone)]
pub struct PathExtractor {
	id: u64,
}

const SELECT_POST: &'static str =
	"SELECT id, title, content, post_time, edit_time, category \
	 FROM posts WHERE id = ?";
const UPDATE_POST: &'static str =
	"UPDATE posts \
	 SET title = ?, content = ?, category = ?, edit_time = ? \
	 WHERE id = ?";

pub fn get(state: State) -> (State, Response) {
	match run_get(&state) {
		Ok(response) => (state, response),
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

pub fn edit_get(state: State) -> (State, Response) {
	let id = PathExtractor::borrow_from(&state).id;
	match run_edit_get(&state, id) {
		Ok(response) => (state, response),
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

pub fn edit_post(mut state: State) -> Box<HandlerFuture> {
	let id = PathExtractor::borrow_from(&state).id;
	let f = Body::take_from(&mut state)
		.concat2()
		.then(move |result| match result {
			Ok(body) => match run_edit_post(&state, body.to_vec(), id) {
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
		title = "Amanda Graven's homepage - Submit article",
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = format!(
			include_str!("../web/submit.html"),
			id = "",
			content = "",
			title = "",
			category = "",
		)
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
	let password = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let session = auth::get_session(&state)?;

	// Get values from POST
	let get = |key: &'static str| -> Result<&String> {
		post.get(key).ok_or(Error::MissingParam(key))
	};
	let title = get("title")?;
	let content = get("content")?;
	let category = get("category")?;

	// Check user permissions
	if session.user != "amanda" {
		return Err(Error::Unauthorized);
	}

	// Insert article into database
	let url = format!(
		"mysql://submit:{}@localhost:3306/amandag",
		password
	);
	let pool = mysql::Pool::new(url)?;
	pool.prep_exec(
		"INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
		(title, content, category),
	)?;

	let content = format!(
		include_str!("../web/index.html"),
		title = "Article submitted",
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = "<article>Article submitted.</article>"
	).into_bytes();
	let response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);

	Ok(response)
}

pub fn run_edit_get(state: &State, id: u64) -> Result<Response> {
	let session = auth::get_session(&state)?;
	let article = {
		let pool =
			mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
		let row = pool.first_exec(SELECT_POST, (id,))?
			.ok_or(Error::InvalidId(id as u64))?;
		// Bind values from row
		let (id, title, content, post_time, edit_time, category) =
			mysql::from_row(row);
		Article {
			id,
			title,
			content,
			post_time,
			edit_time,
			category,
			comment_count: 0,
		}
	};
	let content = format!(
		include_str!("../web/index.html"),
		title = format!("Edit article: {}", article.title),
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = format!(
			include_str!("../web/submit.html"),
			id = article.id,
			content = article.content.encode_html(),
			title = article.title.encode_html(),
			category = article.category.encode_html(),
		),
	).into_bytes();
	let response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);
	Ok(response)
}

fn run_edit_post(state: &State, post: Vec<u8>, id: u64) -> Result<Response> {
	let post = cgi::request_decode(post);
	let password = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let session = auth::get_session(&state)?;

	// Get values from POST
	let get = |key: &'static str| -> Result<&String> {
		post.get(key).ok_or(Error::MissingParam(key))
	};
	let title = get("title")?;
	let content = get("content")?;
	let category = get("category")?;

	// Check user permissions
	if session.user != "amanda" {
		return Err(Error::Unauthorized);
	}

	// Insert article into database
	let url = format!(
		"mysql://submit:{}@localhost:3306/amandag",
		password
	);
	let pool = mysql::Pool::new(url)?;
	pool.prep_exec(
		UPDATE_POST,
		(title, content, category, time::get_time(), id),
	)?;

	let content = format!(
		include_str!("../web/index.html"),
		title = "Article submitted",
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = "<article>Article submitted.</article>"
	).into_bytes();
	let response = create_response(
		&state,
		StatusCode::Ok,
		Some((content, mime::TEXT_HTML)),
	);

	Ok(response)
}
