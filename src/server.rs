
/// A macro to easely create an http route.
/// 
/// ## Example
/// ```ignore
/// request_handler! {
/// 	async fn name(req: Request, any_data) -> Result<Response, Error> {}
/// }
/// ```
#[macro_export]
macro_rules! request_handler {
	// handle request without data type
	(
		async fn $name:ident( $($ptt:tt)* ) $($tt:tt)*
	) => (
		$crate::request_handler! {
			async fn $name<Data>($($ptt)*) $($tt)*
		}
	);
	// handle request without data
	(
		async fn $name:ident<$data_ty:ty>($req:ident: $req_ty:ty) $($tt:tt)*
	) => (
		$crate::request_handler! {
			async fn $name<$data_ty>($req: $req_ty, ) $($tt)*
		}
	);
	// final handler
	(
		async fn $name:ident<$data_ty:ty>(
			$req:ident: $req_ty:ty,
			$($data:ident),*
		) -> $ret_ty:ty $block:block
	) => (
		$crate::custom_request_handler!{
			async fn $name<$data_ty>(
				req: $req_ty,
				raw_data
			) -> $ret_ty {
				use $crate::http::header::{Method as __Method};
				use $crate::request::Request as __Request;

				let method = <$req_ty as __Request>::METHOD;
				type __Error = <$req_ty as __Request>::Error;

				// let 
				// deserialize the request
				let $req: $req_ty = if method == __Method::Get {
					$crate::serde_json::from_str("null")
						.map_err(|e| __Error::request(
							format!("malformed request: {}", e)
						))?
				} else {
					req.deserialize().await
						.map_err(|e| __Error::request(
							format!("malformed request: {}", e)
						))?
				};

				macro_rules! __handle_data {
					(header, $req2:ident, $raw_data:ident) => (
						$req2.header()
					);
					($data2:ident, $req2:ident, $raw_data:ident) => (
						$raw_data.$data2()
					);
				}

				#[allow(unused_variables)]
				let raw_data = raw_data;

				$(
					let $data = __handle_data!($data, req, raw_data);
				)*

				async { $block }.await
			}
		}
	)
}

/// A macro to easely create an http route.
/// 
/// where req does not contain the parsed json but a whole fire::request::Request
/// this allows to have a different body than the json but have the benefit of
/// the api structure of error reporting etc...
/// 
/// ## Example
/// ```ignore
/// request_handler! {
/// 	async fn name(req: Request, any_data) -> Result<Response, Error> {}
/// }
/// ```
#[macro_export]
macro_rules! custom_request_handler {
	// handle request without data type
	(
		async fn $name:ident( $($ptt:tt)* ) $($tt:tt)*
	) => (
		$crate::custom_request_handler! {
			async fn $name<Data>($($ptt)*) $($tt)*
		}
	);
	// handle request without data
	(
		async fn $name:ident<$data_ty:ty>($req:ident: $req_ty:ty) $($tt:tt)*
	) => (
		$crate::custom_request_handler! {
			async fn $name<$data_ty>($req: $req_ty, ) $($tt)*
		}
	);
	// final handler
	(
		async fn $name:ident<$data_ty:ty>(
			$req:ident: $req_ty:ty,
			$($data:ident),*
		) -> $ret_ty:ty $block:block
	) => (

		#[allow(non_camel_case_types)]
		struct $name;

		impl $crate::http::routes::Route<$data_ty> for $name {

			fn check(
				&self,
				req: &$crate::http::header::RequestHeader
			) -> bool {
				let method = <$req_ty as $crate::request::Request>::METHOD;
				let path = <$req_ty as $crate::request::Request>::PATH;

				req.method() == &method &&
				$crate::http::routes::check_static(req.uri().path(), path)
			}

			fn call<'a>(
				&'a self,
				req: &'a mut $crate::http::request::Request,
				raw_data: &'a $data_ty
			) -> $crate::http::util::PinnedFuture<
					'a,
					$crate::http::Result<$crate::http::response::Response>
				>
			{
				use $crate::http::body::Body as __Body;
				use $crate::http::header::{StatusCode as __Status};
				use $crate::error::ApiError;
				use $crate::request::Request as __Request;

				type __Response = <$req_ty as __Request>::Response;
				type __Error = <$req_ty as __Request>::Error;

				async fn __handle(
					req: &mut $crate::http::request::Request,
					_raw_data: &$data_ty
				) -> std::result::Result<__Body, __Error> {

					let headers = <$req_ty as __Request>::HEADERS;
					let size_limit = <$req_ty as __Request>::SIZE_LIMIT;
					let timeout = <$req_ty as __Request>::TIMEOUT;

					req.set_timeout(std::time::Duration::from_secs(timeout as u64));
					req.set_size_limit(size_limit);

					if !$crate::util::headers_exist(req.header(), headers) {
						return Err(__Error::request(format!(
							"maformed request: missing headers {:?}",
							headers
						)))
					}

					let $req = req;

					macro_rules! __handle_data {
						(header, $req2:ident, $raw_data:ident) => (
							$req2.header()
						);
						(raw_data, $req2:ident, $raw_data:ident) => (
							$raw_data
						);
						($data2:ident, $req2:ident, $raw_data:ident) => (
							$raw_data.$data2()
						);
					}

					$(
						let $data = __handle_data!($data, $req, _raw_data);
					)*

					let ret: $ret_ty = async { $block }.await;
					let ret: std::result::Result<__Response, __Error> = ret;

					__Body::serialize(&ret?)
						.map_err(|e| __Error::internal(
							format!("malformed response: {}", e)
						))
				}

				$crate::http::util::PinnedFuture::new(async move {

					let ret = __handle(req, raw_data).await;

					let (status, body) = match ret {
						Ok(b) => (__Status::Ok, b),
						Err(e) => {
							$crate::util::request_handle_error(&e);

							let status = e.status_code();
							// now serialize the error
							let body = __Body::serialize(&e)
								.map_err(|e| $crate::http::Error::new(
									$crate::http::error::ServerErrorKind::
										InternalServerError,
									e
								))?;

							(status, body)
						}
					};

					let resp = $crate::http::response::Response::builder()
						.status_code(status)
						.content_type($crate::http::header::Mime::Json)
						.body(body)
						.build();

					Ok(resp)
				})
			}

		}

	)
}



#[cfg(test)]
mod tests {

	use crate::request::{Request, Method};
	use crate::error::{self, ApiError, StatusCode};

	use std::fmt;

	use serde::{Serialize, Deserialize};

	struct Data;

	#[derive(Debug, Serialize, Deserialize)]
	struct RequestData {
		hello: u64
	}

	#[derive(Debug, Serialize, Deserialize)]
	struct ResponseData {
		hi: u64
	}

	impl Request for RequestData {
		type Response = ResponseData;
		type Error = Error;
		const PATH: &'static str = "/api/test";
		const METHOD: Method = Method::Get;
	}

	#[derive(Debug, Serialize)]
	enum Error {}

	impl fmt::Display for Error {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			fmt::Debug::fmt(self, f)
		}
	}

	impl ApiError for Error {
		fn internal<E: error::Error>(_error: E) -> Self { todo!() }
		fn request<E: error::Error>(_error: E) -> Self { todo!() }

		fn status_code(&self) -> StatusCode { todo!() }
	}

	request_handler! {
		async fn test(req: RequestData) -> Result<ResponseData, Error> {
			Ok(ResponseData {
				hi: req.hello
			})
		}
	}

}