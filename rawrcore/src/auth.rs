pub mod authenticators {
    use std::collections::HashMap;

    use reqwest::StatusCode;

    use crate::{errors::RawrCoreError, requestor::Requestor};

    pub trait Authenticator {
        fn auth(&self) -> (String, String);

        fn post(
            &self,
            url: String,
            success_status: Option<StatusCode>,
            data: HashMap<String, String>,
        ) -> reqwest::Response;

        fn authorize_url(
            &self,
            duration: String,
            scopes: Vec<String>,
            state: String,
            implicit: bool,
        ) -> ();

        fn revoke_token(&self, token: String, token_type: Option<String>) -> ();
    }

    struct BaseAuthenticator {
        requestor: Requestor,
        client_id: String,
        redirect_uri: Option<String>,
    }

    impl BaseAuthenticator {
        pub fn new(requestor: Requestor, client_id: String, redirect_uri: Option<String>) -> Self {
            Self {
                requestor,
                client_id,
                redirect_uri,
            }
        }
    }

    impl Authenticator for BaseAuthenticator {}

    struct TrustedAuthenticator {
        base_authenticator: BaseAuthenticator,
        client_secret: String,
    }

    impl TrustedAuthenticator {
        pub fn new(base_authenticator: BaseAuthenticator, client_secret: String) -> Self {
            Self {
                base_authenticator,
                client_secret,
            }
        }
    }
}

pub mod authorizers {
    use std::{collections::HashSet, time};

    use crate::errors::RawrCoreError;

    use super::authenticators::Authenticator;

    pub trait Authorizer {
        fn is_valid(&self) -> bool;
        fn revoke(&self) -> Result<(), RawrCoreError>;
    }

    pub struct BaseAuthorizer<'a> {
        authenticator: &'a dyn Authenticator,
        expiration_timestamp: Option<f32>,
        access_token: Option<String>,
        scopes: Option<HashSet<String>>,
    }

    impl<'a> BaseAuthorizer<'a> {
        pub fn new(authenticator: &'a dyn Authenticator) -> Self {
            Self {
                authenticator,
                expiration_timestamp: None,
                access_token: None,
                scopes: None,
            }
        }

        pub fn clear_access_token(&mut self) {
            self.expiration_timestamp = None;
            self.access_token = None;
            self.scopes = None;
        }

        pub fn request_token(&mut self) {}

        pub fn validate_authenticator(&self) {}
    }

    impl <'a> Authorizer for BaseAuthorizer<'a> {
        fn is_valid(&self) -> bool {
            self.access_token.is_some()
                && self.expiration_timestamp.is_some()
                && self.expiration_timestamp.unwrap()
                    < time::SystemTime::now()
                        .duration_since(time::UNIX_EPOCH)
                        .expect("time went backwards :(")
                        .as_millis() as f32
        }

        fn revoke(&self) -> Result<(), RawrCoreError> {
            if self.access_token.is_none() {
                return Err(RawrCoreError::InvalidInvocation("no token available to revoke"));
            }

            self.authenticator.revoke_token(self.access_token.unwrap(), Some("access_token".to_string()));
            self.clear_access_token();

            Ok(())
        }
    }
}
