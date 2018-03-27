extern crate amandag;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;

use amandag::{auth, captcha, cgi};

const RESERVED: &[&str] = &["guest", "admin"];

error_chain! {
	foreign_links {
		Io(std::io::Error);
		Utf8(std::string::FromUtf8Error);
	}
	links {
		Auth(auth::Error, auth::ErrorKind);
		Captcha(captcha::Error, captcha::ErrorKind);
		Cgi(cgi::Error, cgi::ErrorKind);
	}
	errors {
		Post {
			description("POST input missing"),
			display("POST method specified, but no input given"),
		}
		Undefined(param: &'static str) {
			description("Missing POST value"),
			display("Missing parameter: {}", param),
		}
		Reserved(name: String) {
			description("This is not a permitted username"),
			display("{} is a reserved username", name),
		}
	}
}

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			content = format!(
				"<article><h1>Error</h1>{}</article>",
				e.to_string()
			),
			head = "",
			userinfo = "",
			title = "Error"
		);
	}
}

fn run() -> Result<()> {
	let session = auth::get_session()?;
	if cgi::request_method_is("POST") {
		result()?;
		Ok(())
	} else {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Sign up",
			content = include_str!("../web/signup.html"),
			head = "<script src='https://www.google.com/recaptcha/api.js'></script>",
			userinfo = cgi::print_user_info(&session.user),
		);
		Ok(())
	}
}

fn result() -> Result<()> {
	let post = cgi::get_post().ok_or(ErrorKind::Post)?;
	let get = |key: &'static str| -> Result<&String> {
		post.get(key)
			.ok_or(ErrorKind::Undefined(key).into())
	};
	let user = get("user")?;
	// Check if username is reserved
	if RESERVED.contains(&user.as_str()) {
		return Err(ErrorKind::Reserved(user.to_owned()).into());
	}
	// Check reCAPTCHA
	let response = get("g-recaptcha-response")?;
	let secret = String::from_utf8(
		File::open("secret/comment-captcha")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;

	captcha::verify(&secret, &response)?;

	let pass = get("password")?;
	let name = get("name")?;

	auth::create(&user, &pass, &name)?;

	let session = auth::login(&user, &pass)?;
	println!(
		"{}\n{}\n\n",
		format!(
			"Set-Cookie: token={}; Secure; SameSite=Strict",
			session.id,
		),
		include_str!("../web/http-headers")
	);
	println!(
		include_str!("../web/index.html"),
		title = "Signup successful",
		head = "",
		userinfo = cgi::print_user_info(&session.user),
		content = format!(
			"<article><h1>Signup successful</h1>Successfully created user {}",
			user
		)
	);
	Ok(())
}
