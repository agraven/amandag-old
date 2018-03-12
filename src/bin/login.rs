extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::{auth, cgi, mysql};

error_chain! {
	foreign_links {
		Sql(mysql::Error);
	}
	links {
		Auth(auth::Error, auth::ErrorKind);
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
	}
}

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			content = format!("<article><h1>Error</h1>{}</article>", e.to_string()),
			head = "",
			userinfo = "",
			title = "Error"
		);
	}
}

fn run() -> Result<()> {
	let session = auth::get_session()?;
	if cgi::request_method_is("POST") {
		let post = cgi::get_post().ok_or(ErrorKind::Post)?;
		let get = |key: &'static str| -> Result<&String> {
			post.get(key).ok_or(ErrorKind::Undefined(key).into())
		};
		let user = get("user")?;
		let pass = get("password")?;

		let session = auth::login(&user, &pass)?;

		println!(
			"{}\n{}\n\n",
			include_str!("../web/http-headers"),
			format!(
				"Set-Cookie: session={}; Secure; SameSite=Strict",
				session.id,
			)
		);
		println!(
			include_str!("../web/index.html"),
			title = "Login successful",
			head = "",
			userinfo = cgi::print_user_info(&user),
			content = format!(
				"<article><h1>Login successful</h1>Successfully logged in as {}",
				user
			)
		);

		return Ok(());
	} else if session.user != "guest" {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Login successful",
			head = "",
			userinfo = cgi::print_user_info(&session.user),
			content = "<article><h1>Already logged in</h1> \
			           To sign in as another user, you have to log out first.</article>",
		);
	} else {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(
			include_str!("../web/index.html"),
			title = "Login",
			content = include_str!("../web/login.html"),
			userinfo = cgi::print_user_info(&session.user),
			head = ""
		);
	}
	Ok(())
}
