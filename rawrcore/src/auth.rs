use std::collections::HashMap;

use crate::requestor::Requestor;


pub struct BaseAuthenticator {
    requestor: Requestor,
    pub client_id: String,
    pub redirect_uri: String,
}

impl BaseAuthenticator {
    pub fn new(requestor: Requestor, client_id: &str, redirect_uri: &str) -> Self {
        Self {
            requestor,
            client_id: client_id.to_string(),
            redirect_uri: client_id.to_string(),
        }
    }

    fn post(&self, url: &str, success_status: Option<u16>, request_data:>) -> () {
        
    }
}