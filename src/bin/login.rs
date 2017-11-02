extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::{auth, cgi};

error_chain! {
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
		println!(include_str!("../web/index.html"),
			content = format!("<article><h1>Error</h1>{}</article>", e.to_string()),
			head = "",
			title = "Error"
		);
	}
}

fn run() -> Result<()> {
	if cgi::request_method_is("POST") {
		let post = cgi::get_post().ok_or(ErrorKind::Post)?;
		let get = |key: &'static str| -> Result<&String> {
			post.get(key).ok_or(ErrorKind::Undefined(key).into())
		};
		let user = get("user")?;
		let pass = get("password")?;

		let token = auth::login(&user, &pass)?;

		println!("{}{}\n",
			include_str!("../web/http-headers"),
			format!(
				"Set-Cookie: token={}; Secure; SameSite=Strict\n\
				 Set-Cookie: user={}; Secure; SameSite=Strict",
				token.hash,
				token.user
			)
		);
		println!(include_str!("../web/index.html"),
			title = "Login successful",
			head = "",
			content = format!(
				"<article><h1>Login successful</h1>Successfully logged in as {}",
				user
			)
		);

		return Ok(());
	}
	println!("{}\n", include_str!("../web/http-headers"));
	println!(include_str!("../web/index.html"),
		title = "Login",
		content = include_str!("../web/login.html"),
		head = ""
	);
	Ok(())
}
