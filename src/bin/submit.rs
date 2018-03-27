extern crate amandag;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;

use amandag::article::Article;
use amandag::auth;
use amandag::captcha;
use amandag::cgi;
use amandag::cgi::Encode;
use amandag::mysql;
use amandag::time;

const SELECT_POST: &'static str =
	"SELECT id, title, content, post_time, edit_time, category \
	 FROM posts WHERE id = ?";

// Error definitions
error_chain! {
	foreign_links {
		Captcha(captcha::Error);
		Io(std::io::Error);
		Sql(mysql::Error);
		Utf8(std::string::FromUtf8Error);
		ParseInt(std::num::ParseIntError);
	}
	links {
		Auth(auth::Error, auth::ErrorKind);
		Cookie(cgi::Error, cgi::ErrorKind);
	}
	errors {
		Param(s: &'static str) {
			description("missing POST parameter"),
			display("Missing POST value: {}", s),
		}
		Unauthorized {
			description("User does not have permission to submit articles"),
			display("You do not have permission to submit articles"),
		}
		InvalidId(id: u64) {
			description("invalid article id"),
			display("Invalid article id: {}", id),
		}
	}
}
use ErrorKind::Param;

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Error",
			head = "",
			userinfo = "",
			content = format!(
				"<article><h1>Internal server error</h1>\
				 The page could not be displayed because of an internal \
				 error: {}</article>",
				e
			),
		);
	}
}

fn run() -> Result<()> {
	let password = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let session = auth::get_session()?;
	// If article was submitted, don't print submisstion form
	if cgi::request_method_is("POST") {
		// Get a map of POST values
		let map = cgi::get_post().expect("Failed to get post values");

		// Get values from POST
		let get = |key: &'static str| -> Result<&String> {
			map.get(key).ok_or(Param(key).into())
		};
		let title = get("title")?;
		let content = get("content")?;
		let category = get("category")?;

		// Check user permissions
		if session.user != "amanda" {
			return Err(ErrorKind::Unauthorized.into());
		}

		// Insert article into database
		let url = format!(
			"mysql://submit:{}@localhost:3306/amandag",
			password
		);
		let pool = mysql::Pool::new(url)?;
		if let Some(id) = cgi::get_get_member("id") {
			let id = id.parse::<u64>()?;
			pool.prep_exec(
                "UPDATE posts SET title = ?, content = ?, category = ?, edit_time = ? WHERE id = ?",
                (title, content, category, time::get_time(), id),
            )?;
		} else {
			pool.prep_exec(
				"INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
				(title, content, category),
			)?;
		}

		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Article submitted",
			head = "",
			userinfo = cgi::print_user_info(&session.user),
			content = "<article>Article submitted.</article>"
		);

		return Ok(());
	}

	// Edit post
	if let Some(id) = cgi::get_get_member("id") {
		let id = id.parse::<u64>()?;
		let article = {
			let pool = mysql::Pool::new(
				"mysql://readonly:1234@localhost:3306/amandag",
			)?;
			let row = pool.first_exec(SELECT_POST, (id,))?
				.ok_or(ErrorKind::InvalidId(id as u64))?;
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
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
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
		)
	} else {
		// Print submission form
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
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
		);
	}

	Ok(())
}
