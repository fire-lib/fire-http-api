
use http::header::{RequestHeader};

pub fn headers_exist(header: &RequestHeader, headers: &[&str]) -> bool {
	!headers.iter().any(|key| {
		header.value(*key).is_none()
	})
}