extern crate amandag;

use amandag::mysql;
use amandag::Article;

fn main() {
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag").unwrap();
	// Select posts from SQL DATABASE
	let selected: Vec<Article> =
		pool.prep_exec(
			"SELECT id, title, content, post_time, edit_time, category \
			FROM posts ORDER BY post_time DESC LIMIT 20",
			()
		)
	.map(|result| {
			// Iterate through rows
			result.map(|x| x.unwrap()).map(|row| {
				let (id, title, content, post_time, edit_time, category) = mysql::from_row_opt(row).expect("Row conversion error");
				// Get amount of comments on post
				let comment_count = mysql::from_row_opt(
					pool.first_exec("SELECT COUNT(*) AS comment_count FROM comments \
					WHERE post_id = ?", (id,)).unwrap().unwrap()
				).unwrap_or(0);
				Article {id, title, content, post_time, edit_time, category, comment_count}
			}).collect()
		}).unwrap();

	// Print document
	let mut articles = String::new();
	for post in selected {
		articles.push_str(&post.display());
	}
	println!("{}\n", include_str!("../web/http-headers"));
	println!(include_str!("../web/index.html"),
		title = "Amanda Graven's homepage",
		content = articles,
	);
}
