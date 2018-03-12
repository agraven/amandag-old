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
        LoggedOut {
            description("Already logged out"),
            display("Already logged out"),
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
	if session.user == "guest" {
		return Err(ErrorKind::LoggedOut.into());
	}
	auth::logout(&session.id)?;
	println!(
        "{}\n{}\n",
        "Set-Cookie: session=0; Max-Age=0",
        include_str!("../web/http-headers"),
    );
	println!(
		include_str!("../web/index.html"),
		head = "",
		content = "<article><h1>Logged out</h1>Successfully logged out</article>",
		userinfo = "",
		title = "Error"
	);
	Ok(())
}
