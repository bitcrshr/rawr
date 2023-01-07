use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::errors::{RawrCoreError, ResponseErrorData, RedirectData, SpecialErrorData};
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

fn response_to_rawrcore_error(response: &Response) -> Option<RawrCoreError> {
    let response_error_data = ResponseErrorData {
        response: *response.to_owned()
    };

    let err = match response.status() {
        StatusCode::INTERNAL_SERVER_ERROR => RawrCoreError::ServerError(response_error_data),

        StatusCode::BAD_REQUEST => RawrCoreError::BadRequest(response_error_data),

        StatusCode::CONFLICT => RawrCoreError::Conflict(response_error_data),

        StatusCode::FOUND => {
            let location = response.headers().get("location");
            let mut path = "";

            if location.is_some() {
                let loc = location.unwrap();

                let val = match loc.to_str() {
                    Ok(v) => v,
                    Err(_) => ""
                };

                path = val
            }

            RawrCoreError::Redirect(RedirectData {
                path: path.to_string(),
                response: *response.to_owned()
            })
        },

        StatusCode::GATEWAY_TIMEOUT => RawrCoreError::ServerError(response_error_data),
        StatusCode::INTERNAL_SERVER_ERROR => RawrCoreError::ServerError(response_error_data),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => {
            let mut try_after: u32;
            
            let special_err_data = SpecialErrorData {
                response: *response.to_owned(),
                message: "".to_string()
            }
        }
    };

    
}

pub struct Session {
}