[![CI](https://github.com/fire-lib/fire-http-api/actions/workflows/ci.yaml/badge.svg)](https://github.com/fire-lib/fire-http-api/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/fire-http-api)](https://crates.io/crates/fire-http-api)
[![docs.rs](https://img.shields.io/docsrs/fire-http-api)](https://docs.rs/fire-http-api)

Make web apis.

## Features
- stream

## Example
```rust no_run
# use fire_http_api as api;
use std::fmt;
use std::sync::{Arc, Mutex};
use api::request_handler;
use api::request::{Method, Request};
use api::error::{ApiError, Error as ErrorTrait, StatusCode};
use api::http as fire;
use fire::data_struct;
use serde::{Serialize, Deserialize};


// -- Type definitions

#[derive(Debug, Clone, Serialize)]
pub enum Error {
	Internal(String),
	Request(String)
}

impl ApiError for Error {
	fn internal<E: ErrorTrait>(e: E) -> Self {
		Self::Internal(e.to_string())
	}

	fn request<E: ErrorTrait>(e: E) -> Self {
		Self::Request(e.to_string())
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::Internal(_) => StatusCode::InternalServerError,
			Self::Request(_) => StatusCode::BadRequest
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NameReq;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Name {
	firstname: String,
	lastname: String
}

impl Request for NameReq {
	type Response = Name;
	type Error = Error;

	const PATH: &'static str = "/name";
	const METHOD: Method = Method::Get;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetNameReq {
	name: Name
}

impl Request for SetNameReq {
	type Response = ();
	type Error = Error;

	const PATH: &'static str = "/name";
	const METHOD: Method = Method::Put;
}

// -- implementations

data_struct! {
	struct Data {
		name: Mutex<Name>
	}
}

request_handler! {
	async fn get_name(req: NameReq, name) -> Result<Name, Error> {
		let lock = name.lock().unwrap();
		Ok(lock.clone())
	}
}

request_handler! {
	async fn set_name(req: SetNameReq, name) -> Result<(), Error> {
		let mut lock = name.lock().unwrap();
		*lock = req.name;

		Ok(())
	}
}

#[tokio::main]
async fn main() {
	let data = Data {
		name: Mutex::new(Name {
			firstname: "Albert".into(),
			lastname: "Einstein".into()
		})
	};

	let mut server = fire::build("0.0.0.0:3000", data)
		.expect("Failed to parse address");

	server.add_route(get_name);
	server.add_route(set_name);

	server.light().await
		.expect("server paniced");
}
```