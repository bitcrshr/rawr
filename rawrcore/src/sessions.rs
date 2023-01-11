use std::{collections::HashMap, thread, time::Duration};
use lazy_static::lazy_static;
use rand::Rng;
use crate::{errors::{RawrCoreError, ResponseErrorData, RedirectData, SpecialErrorData}, auth::authorizers::Authorizer};
use reqwest::{StatusCode, Response, header::HeaderValue};

lazy_static! {
    static ref RETRY_STATUSES: [u16; 7] = [
        520,
        522,
        StatusCode::BAD_GATEWAY.as_u16(),
        StatusCode::GATEWAY_TIMEOUT.as_u16(),
        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        StatusCode::REQUEST_TIMEOUT.as_u16(),
        StatusCode::SERVICE_UNAVAILABLE.as_u16()
    ];
}

fn is_success_status(response: &Response) -> bool {
    match response.status() {
        StatusCode::ACCEPTED => true,
        StatusCode::CREATED => true,
        StatusCode::OK => true,
        _ => false
    }
}

fn handle_redirect(response: &Response) -> Option<RawrCoreError> {
    let location = response.headers().get("location");
    let mut path = "";

    if location.is_some() {
        let loc = location.unwrap();
        let val = match loc.to_str() {
            Ok(v) => v,
            Err(_) => ""
        };

        path = val;
    }

    Some(RawrCoreError::Redirect(RedirectData {
        path: path.to_string(),
        response: response,
    }))
}

fn response_to_rawrcore_error(response: &Response) -> Option<RawrCoreError> {
    let response_error_data = ResponseErrorData {
        response: response,
    };

    let err: Option<RawrCoreError> = match response.status() {
        StatusCode::INTERNAL_SERVER_ERROR => Some(RawrCoreError::ServerError(response_error_data)),

        StatusCode::BAD_REQUEST => Some(RawrCoreError::BadRequest(response_error_data)),

        StatusCode::CONFLICT => Some(RawrCoreError::Conflict(response_error_data)),

        StatusCode::FOUND => handle_redirect(response),

        StatusCode::GATEWAY_TIMEOUT => Some(RawrCoreError::ServerError(response_error_data)),
        StatusCode::INTERNAL_SERVER_ERROR => Some(RawrCoreError::ServerError(response_error_data)),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => {
            let mut try_after: u16 = 0;
            let try_after_header = response.headers().get("x-try-after");

            if try_after_header.is_some() {
               try_after = match try_after_header {
                Some(ta) => {
                    let ta_string = match ta.to_str() {
                        Ok(s) => s.to_string(),
                        Err(e) => "0".to_string()
                    };

                    match ta_string.parse::<u16>() {
                        Ok(r) => r,
                        Err(_) => 0
                    }
                },

                None => {
                    0
                }
               }
            }
            
            let special_err_data = SpecialErrorData {
                response: response,
                message: "".to_string(),
                retry_after: try_after
            };

            Some(RawrCoreError::SpecialError(special_err_data))
        },
        StatusCode::MOVED_PERMANENTLY => handle_redirect(response),
        StatusCode::NOT_FOUND => Some(RawrCoreError::NotFound(response_error_data)),
        StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => Some(RawrCoreError::TooLarge(response_error_data)),
        StatusCode::URI_TOO_LONG => Some(RawrCoreError::URITooLong(response_error_data)),
        StatusCode::SERVICE_UNAVAILABLE => Some(RawrCoreError::ServerError(response_error_data)),
        StatusCode::TOO_MANY_REQUESTS => Some(RawrCoreError::TooManyRequests(response_error_data)),
        StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS =>Some( RawrCoreError::UnavailableForLegalReasons(response_error_data)),
        code => {
            match code.as_u16() {
                520 => Some(RawrCoreError::ServerError(response_error_data)),
                522 => Some(RawrCoreError::ServerError(response_error_data)),
                _ => None
            }
        }
        
    };

    err
}
trait RetryStrategy {
    fn sleep(&self) -> ();
}

struct FiniteRetryStrategy {
    retries: u8,
}

impl FiniteRetryStrategy {
    fn new(retries: Option<u8>) -> Self {
        Self {
            retries: match retries {
                Some(r) => r,
                None => 3
            }
        }
    }

    fn sleep_seconds(&self) -> Option<f32> {
        if self.retries < 3 {
        let base: f32 = if self.retries == 2 {
                0.0
            } else {
                2.0
            };

            let mut rng = rand::thread_rng();

            return Some(base + 2.0 * rng.gen_range(0.0..1.0));
        }

        None
    }

    fn consume_available_retry(&self) -> Self {
        Self {
            retries: if self.retries == 0 { 0 } else { self.retries - 1 },
        }
    }

    fn should_return_on_failure(&self) -> bool {
        self.retries > 1
    }
}

impl RetryStrategy for FiniteRetryStrategy {
    fn sleep(&self) {
        let sleep_seconds = self.sleep_seconds();

        match sleep_seconds {
            Some(secs) => {
                thread::sleep(Duration::from_secs_f32(secs))
            },
            None => ()
        }
    }
}

pub struct Session {
    authorizer: dyn Authorizer,    
}

impl Session {
    
}