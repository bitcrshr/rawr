
pub struct RequestTokenResponse {
    pub expires_in: i32,
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scope: String
}