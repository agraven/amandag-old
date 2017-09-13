extern crate amandag;

use amandag::cgi;
use amandag::Comment;
use amandag::CommentList;
use amandag::mysql;
use amandag::Post;
use amandag::time;

fn main() {
	// Get map of GET request and get id
	let id: i64 = cgi::get_get_member(String::from("id"))
		.unwrap_or(String::new()).parse().unwrap_or(-1);

	// Establish connection to MySQL server
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")
		.expect("Failed to connect to database");
	// Get first (and hopefully only) article result
	let post: Post = if let Some(row) =
		pool.first_exec(
			"SELECT id, title, content, post_time, edit_time, category \
				FROM posts WHERE id = ?",
			(id,)
		).expect("Failed to get data from database") {
			// Bind values from row
			let (id, title, content, post_time, edit_time, category) =
				mysql::from_row_opt(row).unwrap_or( (
					0,
					String::from("Error!"),
					String::from("Failed to display article: \
						Error while fetching from database"),
					time::get_time(), time::get_time(),
					String::from("Error")
				));
			// Get amount of comments
			let comment_count = if let Some(row) =
				pool.first_exec(
					"SELECT COUNT(*) AS comment_count \
						FROM comments WHERE post_id = ?",
					(id,)
				).unwrap() {
					mysql::from_row_opt(row).unwrap_or(0)
				} else { 0 };
			Post { id, title, content, post_time, edit_time, category, comment_count }
		} else {
			Post {
				id: 0,
				title: String::from("Invalid id"),
				content:
					String::from("Failed to display article: \
					An article was requested with an id that doesn't exist."),
				post_time: time::get_time(),
				edit_time: time::get_time(),
				category: String::from("Error message"),
				comment_count: 0,
			}
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
    }).unwrap();

	// print document
	println!(include_str!("../web/view.html"),
		title = post.title,
		post = post.id,
		article = post.display(),
		comments = comments.display(),
		lbrace = "{",
		rbrace = "}"
	);
}
