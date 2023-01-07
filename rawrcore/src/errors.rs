pub enum RawrCoreError<'r> {
    InvalidInvocation,
    RequestError(RequestErrorData),
    ResponseError(ResponseErrorData<'r>),
    OAuthError(OAuthErrorData<'r>),
    BadJSON(ResponseErrorData<'r>),
    BadRequest(ResponseErrorData<'r>),
    Conflict(ResponseErrorData<'r>),
    Forbidden(ResponseErrorData<'r>),
    InsufficientScope(ResponseErrorData<'r>),
    InvalidToken(ResponseErrorData<'r>),
    NotFound(ResponseErrorData<'r>),
    Redirect(RedirectData<'r>),
    ServerError(ResponseErrorData<'r>),
    SpecialError(SpecialErrorData<'r>),
    TooLarge(ResponseErrorData<'r>),
    TooManyRequests(ResponseErrorData<'r>),
    UnavailableForLegalReasons(ResponseErrorData<'r>),
    URITooLong(ResponseErrorData<'r>)
}

pub struct RequestErrorData {
    pub original_error: reqwest::Error,
    pub request: reqwest::Request
}

pub struct ResponseErrorData<'r> {
    pub response: &'r reqwest::Response,
}

pub struct OAuthErrorData<'r> {
    pub response: &'r reqwest::Response,
    pub error: String,
    pub description: Option<String>
}

pub struct RedirectData<'r> {
    pub response: &'r reqwest::Response,
    pub path: String
}

pub struct SpecialErrorData<'r> {
    pub response: &'r reqwest::Response,
    pub retry_after: u16,
    pub message: String
}