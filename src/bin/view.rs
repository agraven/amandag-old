extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::cgi;
use amandag::Comment;
use amandag::CommentList;
use amandag::mysql;
use amandag::Article;

error_chain! {
    foreign_links {
        SqlConversion(mysql::FromRowError);
        Sql(mysql::Error);
        ParseInt(std::num::ParseIntError);
    }
    errors {
        InvalidId(id: u64) {
            description("invalid article id"),
            display("Invalid article id: {}", id),
        }
        MissingParam(s: &'static str) {
            description("missing POST parameter"),
            display("Missing POST value: {}", s),
        }
    }
}


fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(include_str!("../web/index.html"),
			title = "Internal server error",
            head = "",
			content = e.to_string()
		);
	}
}

fn run() -> Result<()> {
	// Get map of GET request and get id
	let id: i64 = cgi::get_get_member(String::from("id"))
		.ok_or(ErrorKind::MissingParam("id"))?.parse()?;

	// Establish connection to MySQL server
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")?;
    // Get article from database
	let article: Article = {
		let row = pool.first_exec(
			"SELECT id, title, content, post_time, edit_time, category \
				FROM posts WHERE id = ?",
			(id,)
		)?.ok_or(ErrorKind::InvalidId(id as u64))?;
		// Bind values from row
		let (id, title, content, post_time, edit_time, category) =
			mysql::from_row(row);
		// Get amount of comments
		let comment_count = mysql::from_row(pool.first_exec(
				"SELECT COUNT(*) AS comment_count \
					FROM comments WHERE post_id = ?",
				(id,)
		)?.unwrap());
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
	println!(include_str!("../web/index.html"),
		title = article.title,
		head = format!("	<script src='https://www.google.com/recaptcha/api.js?render=explicit&onload=captchaLoad' async defer></script>
	<script>var id = {};</script>
	<script src='/view.js'></script>", article.id),
		content = format!("{}{}{}",
            article.display(),
            include_str!("../web/comment-form.html"),
            comments.display()),
	);
	Ok(())
}
