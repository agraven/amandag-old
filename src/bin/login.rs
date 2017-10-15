extern crate amandag;
#[macro_use]
extern crate error_chain;

use amandag::{auth, cgi};

error_chain! {
	links {
		Auth(auth::Error, auth::ErrorKind);
        Cgi(cgi::Error, cgi::ErrorKind);
	}
}

fn main() {
	if let Err(e) = run() {
		println!("{}\n", include_str!("../web/http-headers"));
		println!(include_str!("../web/index.html"),
			content = e.to_string(),
			head = "",
			title = "Error"
		);
	}
}

fn run() -> Result<()> {
    if cgi::request_method_is("POST") {
        let post = cgi::get_post().unwrap();
        let user = post.get("user").unwrap();

		return Ok(());
    }
    println!("{}\n", include_str!("../web/http-headers"));
    println!(include_str!("../web/login.html"));
	Ok(())
}
