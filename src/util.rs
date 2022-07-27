
use std::fmt;

use tracing::error;
use http::header::{RequestHeader};

pub fn headers_exist(header: &RequestHeader, headers: &[&str]) -> bool {
	!headers.iter().any(|key| {
		header.value(*key).is_none()
	})
}

/// we need to expose this instead of inlining it in the macro since
/// tracing logs the crate name and we wan't it to be associated with
/// fire http instead of the crate that uses the macro
pub fn request_handle_error(e: impl fmt::Debug) {
	error!("request handle error: {:?}", e);
}

macro_rules! trace {
	($($tt:tt)*) => (
		#[cfg(feature = "trace")]
		{
			tracing::trace!($($tt)*);
		}
	)
}