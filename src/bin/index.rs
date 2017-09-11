extern crate amandag;

use amandag::mysql;
use amandag::post::Post;
use amandag::strings;
use amandag::time;

fn main() {
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag").unwrap();
	// Select posts from SQL DATABASE
	let selected_posts: Vec<Post> =
		pool.prep_exec(
			"SELECT id, title, content, post_time, edit_time, category \
			FROM posts ORDER BY post_time DESC LIMIT 20",
			()
		)
	.map(|result| {
			// Iterate through rows
			result.map(|x| x.unwrap()).map(|row| {
				let (id, title, content, post_time, edit_time, category) = mysql::from_row_opt(row)
					.unwrap_or((
                        0,
						String::from("Error!"),
						String::from("Failed to display article: Error while fetching from database"),
						time::get_time(), time::get_time(),
						String::from("Error")
					));
				// Get amount of comments on post
				let comment_count = mysql::from_row_opt(
					pool.first_exec("SELECT COUNT(*) AS comment_count FROM comments \
					WHERE post_id = ?", (id,)).unwrap().unwrap()
				).unwrap_or(0);
				Post {id, title, content, post_time, edit_time, category, comment_count}
			}).collect()
		}).unwrap();

	// Print document
	println!("{}", strings::format_document_header("Amanda Graven's Homepage"));
	for post in selected_posts {
		println!("{}", post.display());
	}
	println!("{}", strings::DOCUMENT_FOOTER);
}
