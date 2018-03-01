extern crate amandag;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;

use amandag::auth;
use amandag::captcha;
use amandag::cgi;
use amandag::mysql;

// Error definitions
error_chain! {
	foreign_links {
		Captcha(captcha::Error);
		Io(std::io::Error);
		Sql(mysql::Error);
		Utf8(std::string::FromUtf8Error);
	}
    links {
        Auth(auth::Error, auth::ErrorKind);
        Cookie(cgi::Error, cgi::ErrorKind);
    }
	errors {
		Param(s: &'static str) {
			description("missing POST parameter"),
			display("Missing POST value: {}", s),
		}
	}
}
use ErrorKind::Param;

/*impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", match *self {
			CaptchaError(ref e) => format!("reCAPTCHA failed: {}", e),
			IoError(ref e) => format!("I/O error: {}", e),
			MissingError(ref s) => format!("Missing parameter '{}'", s),
			SqlError(ref e) => format!("Database error: {}", e),
			Utf8Error(ref e) => format!("Invalid UTF-8: {}", e),
		})?;
		Ok(())
	}
}*/

fn main() {
	if let Err(e) = run() {
        println!(include_str!("../web/index.html"),
            title = "Error:",
            head = "",
            content = format!("<article><h1>Internal server error</h1>\
                The page could not be displayed because of an internal \
                error: {}</article>", e),
        );
    }
}

fn run() -> Result<()> {
	// If article was submitted, don't print submisstion form
	if cgi::request_method_is("POST") {
		// Get a map of POST values
		let map = cgi::get_post().expect("Failed to get post values");

		// Get values from POST
		let get = |key: &'static str| -> Result<&String> {
			map.get(key).ok_or(Param(key).into())
		};
		let title = get("title")?;
		let content = get("content")?;
		let category = get("category")?;
		let response = get("g-recaptcha-response")?;
		let secret = String::from_utf8(
			File::open("secret/submit-captcha")?
				.bytes()
				.map(|b| b.unwrap())
				.collect()
		)?;
        let password = String::from_utf8(
			File::open("secret/db-submit")?
				.bytes()
				.map(|b| b.unwrap())
				.collect()
		)?;
        // Verify user is authenticated
        let cookies = cgi::get_cookies()?;
        let session = cookies.get("session").ok_or::<Error>(Param("session").into())?;
        if let Err(e) = auth::auth(session) {
            bail!(e)
        }

		// Verify captcha
		captcha::verify(&secret, &response)?;

		// Insert article into database
		let url = format!("mysql://submit:{}@localhost:3306/amandag", password);
		mysql::Pool::new(url)?
			.prep_exec(
				"INSERT INTO posts (title, content, category) VALUES (?, ?, ?)",
				(title, content, category)
			)?;

		println!(include_str!("../web/index.html"),
			title = "Article submitted",
            head = "",
			content = "<article>Article submitted.</article>"
		);

		return Ok(());
	}
	// Print submission form
	println!("{}\n", include_str!("../web/http-headers"));
	println!(include_str!("../web/index.html"),
		title = "Amanda Graven's homepage - Submit article",
		head = "",
		content = include_str!("../web/submit.html")
	);

	Ok(())
}
