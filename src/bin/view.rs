#[macro_use]
extern crate amandag;

use std::fmt::{self, Display, Formatter};

use amandag::cgi;
use amandag::Comment;
use amandag::CommentList;
use amandag::mysql;
use amandag::Article;

// Error handling
enum Error {
	FromRowError(mysql::FromRowError),
	MysqlError(mysql::Error),
	ParseIntError(std::num::ParseIntError),
    InvalidIdError(u64),
    MissingError(&'static str),
}

impl_error![
	(FromRowError, mysql::FromRowError),
	(MysqlError, mysql::Error),
	(ParseIntError, std::num::ParseIntError)
];

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		use Error::*;
		write!(f, "{}", match *self {
			FromRowError(ref e) => format!("Database error: {}", e),
			MysqlError(ref e) => format!("Database error: {}", e),
			ParseIntError(ref e) => format!("Invalid article id: {}", e),
            InvalidIdError(ref id) => format!("No article with id {}", id),
            MissingError(ref p) => format!("Missing parameter: {}", p),
		})?;
		Ok(())
	}
}


fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(include_str!("../web/index.html"),
			title = "Internal server error",
			content = e.to_string()
		);
	}
}

fn run() -> Result<(), Error> {
	// Get map of GET request and get id
	let id: i64 = cgi::get_get_member(String::from("id"))
		.ok_or(Error::MissingError("id"))?.parse()?;

	// Establish connection to MySQL server
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
    // Get article from database
	let article: Article = {
		let row = pool.first_exec(
			"SELECT id, title, content, post_time, edit_time, category \
				FROM posts WHERE id = ?",
			(id,)
		)?.ok_or(Error::InvalidIdError(id as u64))?;
		// Bind values from row
		let (id, title, content, post_time, edit_time, category) =
			mysql::from_row(row);
		// Get amount of comments
		let comment_count = mysql::from_row_opt(pool.first_exec(
				"SELECT COUNT(*) AS comment_count \
					FROM comments WHERE post_id = ?",
				(id,)
		)?.unwrap())?;
		Article { id, title, content, post_time, edit_time, category, comment_count }
	};

	let comments: Vec<Comment> =
		pool.prep_exec(
			"SELECT id, author, content, post_time, parent_id \
				FROM comments WHERE post_id = ?",
			(id,)
		).map(|result| { result.map(|x| x.unwrap()).map(|row| {
			let (id, author, content, post_time, parent_id) = mysql::from_row(row);
			Comment {id, author, content, post_time, parent_id}
		}).collect()
	})?;

	// print document
	println!("{}\n", include_str!("../web/http-headers"));
	println!(include_str!("../web/view.html"),
		title = article.title,
		id = article.id,
		article = article.display(),
		comments = comments.display(),
	);
	Ok(())
}
