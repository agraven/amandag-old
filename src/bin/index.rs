extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::Article;
use amandag::auth;
use amandag::cgi;
use amandag::mysql;

const SELECT_COMMENT_COUNT: &'static str = "SELECT COUNT(*) AS comment_count \
    FROM comments WHERE post_id = ?";
const SELECT_ARTICLES: &'static str = "SELECT id, title, content, post_time, edit_time, category \
	 FROM posts \
	 ORDER BY post_time DESC \
	 LIMIT 20";

error_chain! {
	foreign_links {
		Sql(mysql::Error);
	}
	links {
		Auth(auth::Error, auth::ErrorKind);
		Cgi(cgi::Error, cgi::ErrorKind);
	}
}

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Amanda's homepage: Error",
			head = "",
			content = e.to_string(),
			userinfo = "",
		);
	}
}

fn run() -> Result<()> {
	let pool =
		mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
	let session = auth::get_session()?;
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
					let comment_count =
						mysql::from_row(
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
	println!("{}\n", include_str!("../web/http-headers"));
	println!(
		include_str!("../web/index.html"),
		userinfo = cgi::print_user_info(&session.user),
		title = "Amanda Graven's homepage",
		head = "",
		content = articles,
	);
	Ok(())
}
