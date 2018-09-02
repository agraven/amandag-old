#![recursion_limit = "128"]
extern crate crypto;
extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hoedown;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate lazy_static;
extern crate mime;
extern crate mysql;
extern crate native_tls;
extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate tokio_core;
extern crate tree_magic;
extern crate url;

mod article;
mod auth;
mod captcha;
mod cgi;
mod comment;
mod db;
mod error;
mod handler;

pub use article::Article;

use gotham::router::Router;
use gotham::router::builder::DefineSingleRoute;
use gotham::router::builder::DrawRoutes;
use gotham::router::builder::build_simple_router;

fn router() -> Router {
	build_simple_router(|route| {
		route
			.get_or_head("/")
			.to(handler::index::handle);

		route
			.get_or_head("/article/:id")
			.with_path_extractor::<handler::view::PathExtractor>()
			.to(handler::view::handle);

		route
			.post("/comment")
			.to(handler::comment::handle);

		route
			.get_or_head("/files/:name")
			.with_path_extractor::<handler::file::PathExtractor>()
			.to(handler::file::handle);

		route
			.get("/submit/:id")
			.with_path_extractor::<handler::submit::PathExtractor>()
			.to(handler::submit::edit_get);
		route
			.post("/submit/:id")
			.with_path_extractor::<handler::submit::PathExtractor>()
			.to(handler::submit::edit_post);
		route.get("/submit").to(handler::submit::get);
		route.post("/submit").to(handler::submit::post);

		route.get("/signup").to(handler::signup::get);
		route.post("/signup").to(handler::signup::post);

		route.get("/login").to(handler::login::get);
		route.post("/login").to(handler::login::post);
		route
			.get_or_head("/logout")
			.to(handler::logout::handle);
	})
}

pub fn main() {
	let addr = "localhost:42069";
	eprintln!("Listening for requests at http://{}", addr);
	gotham::start(addr, router())
}
