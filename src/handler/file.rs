use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Response, StatusCode};

use mime;

use tree_magic;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use error;
use error::Result;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct PathExtractor {
	name: String,
}

// TODO: error handling (separate run fn)
pub fn handle(state: State) -> (State, Response) {
	match run(&state) {
		Ok(response) => (state, response),
		Err(e) => {
			let content = error::print(e).into_bytes();
			let response = create_response(
				&state,
				StatusCode::InternalServerError,
				Some((content, mime::TEXT_HTML)),
			);
			(state, response)
		}
	}
}

fn run(state: &State) -> Result<Response> {
	let path: PathBuf = [
		"files",
		&PathExtractor::borrow_from(&state).name,
	].iter()
		.collect();
	// Check if file exists
	if !path.is_file() {
		let response = create_response(
			&state,
			StatusCode::NotFound,
			Some((
				String::from("404 File not found").into_bytes(),
				mime::TEXT_PLAIN,
			)),
		);
		return Ok(response);
	}
	// Get MIME type
	let mut mime: mime::Mime = tree_magic::from_filepath(&path)
		.parse()
		.unwrap_or(mime::APPLICATION_OCTET_STREAM);
	if path.extension() == Some(OsStr::new("css")) {
		mime = mime::TEXT_CSS;
	}
	if path.extension() == Some(OsStr::new("js")) {
		mime = mime::TEXT_JAVASCRIPT;
	}
	// Read file content
	let content = File::open(&path)?
		.bytes()
		.map(|b| b.unwrap())
		.collect();
	let response =
		create_response(&state, StatusCode::Ok, Some((content, mime)));
	//let date = path.metadata()?.created()?;
	/*response
		.headers_mut()
		.set(header::Date(date.into()));*/
	Ok(response)
}
