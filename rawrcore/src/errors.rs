pub enum RawrCoreError {
    InvalidInvocation,
    RequestError(RequestErrorData),
    ResponseError(ResponseErrorData),
    OAuthError(OAuthErrorData),
    BadJSON(ResponseErrorData),
    BadRequest(ResponseErrorData),
    Conflict(ResponseErrorData),
    Forbidden(ResponseErrorData),
    InsufficientScope(ResponseErrorData),
    InvalidToken(ResponseErrorData),
    NotFound(ResponseErrorData),
    Redirect(RedirectData),
    ServerError(ResponseErrorData),
    SpecialError(SpecialErrorData),
    TooLarge(ResponseErrorData),
    TooManyRequests(ResponseErrorData),
    UnavailableForLegalReasons(ResponseErrorData),
    URITooLong(ResponseErrorData)
}

pub struct RequestErrorData {
    pub original_error: reqwest::Error,
    pub request: reqwest::Request
}

pub struct ResponseErrorData {
    pub response: reqwest::Response,
}

pub struct OAuthErrorData {
    pub response: reqwest::Response,
    pub error: String,
    pub description: Option<String>
}

pub struct RedirectData {
    pub response: reqwest::Response,
    pub path: String
}

pub struct SpecialErrorData {
    pub response: reqwest::Response,
    pub retry_after: u16,
    pub message: String
}