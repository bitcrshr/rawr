use crate::{errors::RawrCoreError, sessions::Session};

pub struct Requestor {
    http: Session,
    pub oauth_url: String,
    pub reddit_url: String,
    pub timeout: f32,
}

impl Requestor {
    pub fn new<'r>(
        user_agent: &str,
        oauth_url: Option<&str>,
        reddit_url: Option<&str>,
        session: Option<Session>,
        timeout: Option<f32>,
    ) -> Result<Self, RawrCoreError<'r>> {
        if user_agent.eq("") || user_agent.len() < 7 {
            return Err(RawrCoreError::InvalidInvocation("user_agent is not descriptive"));
        }

        Ok(Self {
            http: match session {
                Some(sess) => sess,
                None => Session {},
            },

            oauth_url: match oauth_url {
                Some(url) => url.to_string(),
                None => "https://oauth.reddit.com".to_string(),
            },

            reddit_url: match reddit_url {
                Some(url) => url.to_string(),
                None => "https://www.reddit.com".to_string(),
            },

            timeout: match timeout {
                Some(to) => to,
                None => *crate::constants::TIMEOUT,
            },
        })
    }

    pub fn close(&self) {}

    pub fn request(&self, timeout: Option<f32>) -> reqwest::Response {

    }
}