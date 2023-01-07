use lazy_static::lazy_static;

lazy_static! {
    pub static ref ACCESS_TOKEN_PATH: String = "/api/v1/access_token".to_string();
    pub static ref AUTHORIZATION_PATH: String = "/api/v1/authorize".to_string();
    pub static ref REVOKE_TOKEN_PATH: String = "/api/v1/revoke_token".to_string();  
    pub static ref TIMEOUT: f32 = 16.0;  
}