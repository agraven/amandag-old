extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::Comment;
use amandag::auth;
use amandag::captcha;
use amandag::cgi;
use amandag::mysql;
use amandag::time;

use std::fs::File;
use std::io::Read;

const SELECT_UNUSED: &str = r#"SELECT min(unused) AS unused
    FROM (
        SELECT MIN(t1.id)+1 as unused
        FROM comments AS t1
        WHERE NOT EXISTS (SELECT * FROM comments AS t2 WHERE t2.id = t1.id+1)
        UNION
        SELECT 1
        FROM DUAL
        WHERE NOT EXISTS (SELECT * FROM comments WHERE id = 1)
    ) AS subquery"#;

error_chain! {
	links {
		Captcha(captcha::Error, captcha::ErrorKind);
		Auth(auth::Error, auth::ErrorKind);
	}
	foreign_links {
		Io(std::io::Error);
		ParseInt(std::num::ParseIntError);
		Sql(mysql::Error);
		Utf8(std::string::FromUtf8Error);
	}
	errors {
		MissingParam(p: &'static str) {
			description("missing POST paramater"),
			display("Missing POST paramater: {}", p),
		}
		Method {
			description("wrong http method"),
			display("Wrong HTTP method, expected POST."),
		}
	}
}

fn main() {
	if let Err(e) = run() {
		panic!(e.to_string())
	}
}

fn run() -> Result<()> {
	let session = auth::get_session()?;
	if !cgi::request_method_is("POST") {
		return Err(ErrorKind::Method.into());
	}
	let post_map = cgi::get_post().unwrap();
	// Get values
	let get = |key: &'static str| -> Result<&String> {
		post_map
			.get(key)
			.ok_or(ErrorKind::MissingParam(key).into())
	};
	let author = get("name")?.clone();
	let content = get("content")?.clone();
	let post_id = get("id")?.parse::<i64>()?;
	let parent_id = get("parent")?.parse::<i64>()?;
	if session.user == "guest" {
		let response = get("g-recaptcha-response")?;
		let secret = String::from_utf8(
			File::open("secret/comment-captcha")?
				.bytes()
				.map(|b| b.unwrap())
				.collect(),
		)?;
		captcha::verify(&secret, &response)?;
	}

	let password = String::from_utf8(
		File::open("secret/password")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;

	let options = format!(
		"mysql://root:{}@localhost:3306/amandag",
		password
	);
	let pool = mysql::Pool::new(options)?;
	// Get a unique id
	let id: u64 = mysql::from_row(pool.first_exec(SELECT_UNUSED, ())?.unwrap());
	pool.prep_exec(
		"INSERT INTO comments (id, user, author, content, post_id, parent_id) \
		 VALUES (?, ?, ?, ?, ?, ?)",
		(
			id,
			&session.user,
			&author,
			&content,
			post_id,
			parent_id,
		),
	)?;
	let post_time = time::get_time();

	println!(
		"Content-Type: text/html; charset=utf-8\n\n{}",
		Comment {
			id,
			author,
			user: session.user.clone(),
			content,
			post_time,
			parent_id,
		}.display(&session.user == "guest")
	);
	Ok(())
}
