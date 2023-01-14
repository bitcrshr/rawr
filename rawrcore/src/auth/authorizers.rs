use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::HashSet;

use crate::{
    constants,
    errors::{OAuthErrorData, RawrCoreError, ResponseErrorData},
    models::RequestTokenResponse,
};

use super::authenticators::Authenticator;

pub struct BaseAuthorizer {
    authenticator: Authenticator,
    expiration_timestamp: Option<DateTime<Utc>>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    scopes: Option<HashSet<String>>,
    authenticator_is_trusted: bool,
}
impl BaseAuthorizer {
    pub fn new(authenticator: impl Authenticator, is_trusted: bool) -> Self {
        Self {
            authenticator,
            expiration_timestamp: None,
            access_token: None,
            scopes: None,
            refresh_token: None,
            authenticator_is_trusted: is_trusted,
        }
    }

    fn clear_access_token(&self) {
        self.expiration_timestamp = None;
        self.access_token = None;
        self.scopes = None;
    }

    async fn request_token<T>(&mut self, data: T) -> Result<(), RawrCoreError>
    where
        T: Serialize,
    {
        let url =
            self.authenticator.get_requestor().reddit_url + constants::ACCESS_TOKEN_PATH.as_str();
        let pre_request_time = Utc::now();
        let response = self.authenticator.post(url.as_str(), None, data);

        if response.is_err() {
            return Err(response.err().unwrap());
        }

        let res = response.unwrap();
        let payload = res
            .json::<RequestTokenResponse>()
            .await
            .expect("Could not parse request_token response to JSON.");

        if payload.error.is_some() {
            return Err(RawrCoreError::OAuthError(OAuthErrorData {
                response: &response,
                error: payload.error.unwrap(),
                description: payload.error_description,
            }));
        }

        self.expiration_timestamp = pre_request_time
            .checked_sub_signed(Duration::seconds(10))
            .unwrap()
            .checked_add_signed(Duration::seconds(payload.expires_in as i64));

        self.access_token = Some(payload.access_token);
        self.refresh_token = payload.refresh_token;
        self.scopes = Some(HashSet::new::<String>());

        for scope in payload.scope.split(" ") {
            self.scopes.get_or_insert(scope)
        }

        Ok(())
    }

    fn validate_authenticator(&self) -> Result<(), RawrCoreError> {
        // TODO: implement? maybe?
        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.access_token.is_some()
            && self.expiration_timestamp.is_some()
            && Utc::now().cmp(&self.expiration_timestamp.unwrap()).is_lt()
    }

    fn revoke(&self) -> Result<(), RawrCoreError> {
        if self.access_token.is_none() {
            return Err(RawrCoreError::InvalidInvocation(
                "no token available to revoke",
            ));
        }

        self.authenticator
            .revoke_token(self.access_token.unwrap().as_str(), Some("access_token"));
        self.clear_access_token();

        Ok(())
    }
}

pub struct Authorizer {
    base: BaseAuthorizer,
    post_refresh_callback: Option<Fn(Authorizer)>,
    pre_refresh_callback: Option<Fn(Authorizer)>,
}
impl Authorizer {
    pub fn new(
        base: BaseAuthorizer,
        post_refresh_callback: Option<Fn(Authorizer)>,
        pre_refresh_callback: Option<Fn(Authorizer)>,
    ) -> Self {
        Self {
            base,
            post_refresh_callback,
            pre_refresh_callback,
        }
    }

    pub async fn authorize(&self, code: &str) -> Result<(), RawrCoreError> {
        if self.base.authenticator.get_redirect_uri().is_none() {
            return Err(RawrCoreError::InvalidInvocation(
                "redirect URI not provided",
            ));
        }

        self.base.request_token([
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.base.authenticator.get_redirect_uri()?),
        ]).await?;

        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), RawrCoreError> {
        if self.pre_refresh_callback.is_some() {
            self.pre_refresh_callback.unwrap()(self);
        }

        if self.base.refresh_token.is_none() {
            return Err(RawrCoreError::InvalidInvocation("refresh token not provided"))
        }

        self.base.request_token(
            [
                ("grant_type", "refresh_token"),
                ("refresh_token", self.base.refresh_token.unwrap().as_str())
            ]
        ).await?;

        if self.post_refresh_callback.is_some() {
            self.post_refresh_callback.unwrap()(self);
        }

        Ok(())
    }

    pub fn revoke(&mut self, only_access: bool) -> Result<(), RawrCoreError> {
        if only_access || self.base.refresh_token.is_none() {
            return self.base.revoke();
        }

        self.base.authenticator.revoke_token(self.base.refresh_token.unwrap().as_str(), Some("refresh_token"))?;
        self.base.clear_access_token()?;
        self.base.refresh_token = None;

        Ok(())
    }
}

pub struct DeviceIdAuthorizer {
    base: BaseAuthorizer,
    device_id: Option<String>,
    scopes: Option<Vec<String>>,
}
impl DeviceIdAuthorizer {
    pub fn new(base: BaseAuthorizer, device_id: Option<String>, scopes: Option<Vec<String>>) -> Self {
        Self {
            base,
            device_id,
            scopes
        }
    }

    pub async fn refresh(&self) -> Result<(), RawrCoreError> {
        let data: Vec<(&str, &str)> = vec![];

        if self.scopes.is_some() {
            let scopes = self.scopes.unwrap();
            data.push(("scope", scopes.join(" ").as_str()));
        }

        let grant_type = "https://oauth.reddit.com/grants/installed_client";
        data.push(("grant_type", grant_type));
        data.push(("device_id", self.device_id.unwrap_or("".to_string()).as_str()));
        
        self.base.request_token(data).await
    }
}

pub struct ImplicitAuthorizer {
    base: BaseAuthorizer,
    access_token: String,
    expiration_timestamp: DateTime<Utc>,
    scopes: HashSet<String>,
}
impl ImplicitAuthorizer {
    fn new(base: BaseAuthorizer, access_token: String, expires_in: u64, scope: String) -> Self {
        let expiration_timestamp = Utc::now().checked_add_signed(Duration::seconds(expires_in)).expect("Could not parse datetime");
        let scopes = HashSet::<String>::new();

        scope.split(" ").for_each(|s| {
            scopes.insert(s.to_string());
        });

        Self {
            base,
            access_token,
            scopes,
            expiration_timestamp
        }
    }
}

struct ReadOnlyAuthorizer {
    authorizer: Authorizer,
    scopes: Option<Vec<String>>,
}
impl ReadOnlyAuthorizer {
    fn new(authorizer: Authorizer, scopes: Option<Vec<String>>) -> Self {
        Self {
            authorizer,
            scopes
        }
    }

    pub async fn refresh(&self) -> Result<(), RawrCoreError> {
        let data: Vec<(&str, &str)> = vec![];

        if self.scopes.is_some() {
            data.push(("scope", self.scopes.unwrap().join(" ").as_str()));
        }

        data.push(("grant_type", "client_credentials"));

        self.request_token(data).await
    }
}

struct ScriptAuthorizer {
    base: BaseAuthorizer,
    username: Option<String>,
    password: Option<String>,
    two_factor_callback: Option<Fn() -> String>,
    scopes: Option<Vec<String>>
}
impl ScriptAuthorizer {
    pub fn new(base: BaseAuthorizer, username: Option<String>, password: Option<String>, two_factor_callback: Option<Fn() -> String>, scopes: Option<Vec<String>>) -> Self {
        Self {
            base,
            username,
            password,
            two_factor_callback,
            scopes
        }
    }

    pub async fn refresh(&self) -> Result<(), RawrCoreError> {
        let data: Vec<(&str, &str)> = vec![];

        if self.scopes.is_some() {
            let scopes = self.scopes.unwrap().join(" ").as_str();
            data.push(("scope", scopes));
        }

        if self.two_factor_callback.is_some() {
            let tfc: Fn() -> String = self.two_factor_callback.unwrap();

            let code = tfc();
            if !code.is_empty() {
                data.push(("otp", code.as_str()));
            }
        }

        self.base.request_token(data).await
    }
}
