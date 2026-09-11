#![forbid(unsafe_code)]

use declmig_validation::{ProblemDetails, RequestMeta};
use garde::Validate;

pub fn validated_request_headers(meta: &RequestMeta) -> Result<Vec<(String, String)>, garde::Report> {
    meta.validate()?;
    let mut headers = vec![("x-request-id".to_owned(), meta.request_id.clone()), ("traceparent".to_owned(), meta.trace_id.clone())];
    if let Some(locale) = &meta.locale { headers.push(("accept-language".to_owned(), locale.clone())); }
    Ok(headers)
}

pub fn validate_problem(problem: &ProblemDetails) -> Result<(), garde::Report> { problem.validate() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_before_transport() {
        let meta = RequestMeta {request_id: "req-1".into(), trace_id: "trace-1".into(), locale: None};
        assert!(validated_request_headers(&meta).is_ok());
    }
}
