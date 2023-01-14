use reqwest::{StatusCode, Response, Url};
use serde::Serialize;

use crate::{requestor::Requestor, errors::{RawrCoreError, ResponseErrorData}, constants};

pub trait Authenticator {    
    fn new(requestor: Requestor, client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> Self;

    fn auth(&self) -> (String, String);

    fn post<T>(&self, url: &str, success_status: Option<StatusCode>, data: T) -> Result<Response, RawrCoreError> where T: Serialize {
        if success_status.is_none() {
            success_status = Some(StatusCode::OK)
        }

        let requestor = self.get_requestor();
        
        // TODO: fixme once requestor is implemented
        let response = requestor.request();

        if response.status().ne(success_status.unwrap()) {
            return Err(RawrCoreError::ResponseError(ResponseErrorData {
                response
            }))
        }

        response
    }

    fn authorize_url(
        &self,
        duration: &str,
        scopes: Vec<String>,
        state: &str,
        implicit: bool
    ) -> Result<String, RawrCoreError> {
        if self.redirect_url.is_none() {
            return Err(RawrCoreError::InvalidInvocation("redirect_uri not provided"))
        }

        if implicit && !self.is_untrusted() {
            return Err(RawrCoreError::InvalidInvocation("Only UntrustedAuthenticator can use the implicit grant flow"))
        }

        if implicit && duration.eq("temporary") {
            return Err(RawrCoreError::InvalidInvocation("The implicit grant flow only supports temporary access tokens"))
        } 

        let requestor = self.get_requestor();
        let url_string =  requestor.reddit_url + constants::AUTHORIZATION_PATH;
        let params = [
            ("client_id", self.get_client_id()),
            ("duration", duration),
            ("redirect_uri", self.get_redirect_uri()),
            ("response_type", match implicit {
                true => "token",
                false => "code"
            }),
            ("scope", scopes.join(" ")),
            ("state", state)
        ];

        match Url::parse_with_params(url_string.as_str(), params) {
            Ok(url) => url.as_str(),
            Err(e) => Err(RawrCoreError::InvalidInvocation("Error creating url in authorize_url"))
        }
    }

    fn revoke_token(&self, token: &str, token_type: Option<&str>) {
        let data = vec![("token", token)];
        if token_type.is_some() {
            data.push(("token_type_hint", token_type))
        }

        let requestor = self.get_requestor();
        let url = requestor.reddit_url + constants::REVOKE_TOKEN_PATH;
        self.post(url.as_str(), None, data);
    }

    fn get_redirect_uri(&self) -> Option<&str>;
    fn get_requestor(&self) -> &Requestor;
    fn get_client_id(&self) -> &str;
    fn get_client_secret(&self) -> &str;
    fn is_untrusted() -> bool;
}

pub struct TrustedAuthenticator<'a> {
    requestor: Requestor,
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: Option<&'a str>,
}
impl <'a> TrustedAuthenticator<'a>  {
    fn new(requestor: Requestor, client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> Self {
        Self {
            requestor,
            client_id,
            client_secret,
            redirect_uri
        }
    }
}
impl <'a> Authenticator for TrustedAuthenticator<'a> {
    fn get_redirect_uri(&self) -> Option<&str> {
        self.redirect_uri
    }

    fn get_requestor(&self) -> &Requestor {
        &self.requestor
    }

    fn get_client_id(&self) {
        self.client_id
    }

    fn get_client_secret(&self) -> &str {
        self.client_secret
    }

    fn auth(&self) -> (String, String) {
        (self.client_id.to_string(), self.client_secret.to_string())
    }
}

pub struct UntrustedAuthenticator<'a> {
    requestor: Requestor,
    client_id: &'a str,
    redirect_uri: Option<&'a str>
}
impl <'a> UntrustedAuthenticator<'a> {
    fn new(requestor: Requestor, client_id: &str, redirect_uri: Option<&str>) -> Self {
        Self {
            requestor,
            client_id,
            redirect_uri
        }
    }
}
impl <'a> Authenticator for UntrustedAuthenticator<'a> {
    fn get_requestor(&self) -> &Requestor {
        &self.requestor
    }

    fn get_client_id(&self) -> &str {
        self.client_id
    }

    fn get_client_secret(&self) -> &str {
        ""
    }

    fn get_redirect_uri(&self) -> Option<&str> {
       self.redirect_uri 
    }
}