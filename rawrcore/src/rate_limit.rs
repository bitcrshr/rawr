use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
use reqwest::{Response, header::HeaderMap};
use serde::Serialize;
use serde_json::Value;
use std::{
    cmp::{max_by, min_by},
    collections::HashMap,
    time::Duration as StdDuration,
};

pub struct RateLimiter {
    pub remaining: Option<ChronoDuration>,
    pub next_request_timestamp: Option<DateTime<Utc>>,
    pub reset_timestamp: Option<DateTime<Utc>>,
    pub used: Option<u32>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            remaining: None,
            next_request_timestamp: None,
            reset_timestamp: None,
            used: None,
        }
    }

    pub fn delay(&self) {
        if self.next_request_timestamp.is_none() {
            return;
        }

        let sleep_seconds =
            self.next_request_timestamp.unwrap().timestamp() - Utc::now().timestamp();

        if sleep_seconds <= 0 {
            return;
        }

        println!("Sleeping: {} seconds prior to call", sleep_seconds);
        std::thread::sleep(StdDuration::from_secs(sleep_seconds as u64))
    }

    pub fn update(&mut self, response_headers: reqwest::header::HeaderMap) {
        if response_headers.get("x-ratelimit-remaining").is_none() {
            match self.remaining {
                Some(remaining) => {
                    self.remaining = remaining.checked_sub(&ChronoDuration::seconds(1));

                    match self.used {
                        Some(used) => self.used = Some(used + 1),

                        None => self.used = Some(1),
                    }
                }

                None => {
                    return;
                }
            }

            return;
        }

        let now = Utc::now();

        let ratelimit_reset = response_headers
            .get("x-ratelimit-reset")
            .expect("no x-ratelimit-reset header present");
        let ratelimit_remaining = response_headers
            .get("x-ratelimit-remaining")
            .expect("no x-ratelimit-remaining header present");
        let ratelimit_used = response_headers
            .get("x-ratelimit-used")
            .expect("no x-ratelimit-used header present");

        let seconds_to_reset = ratelimit_reset
            .to_str()
            .expect("could not parse x-ratelimit-reset header to &str")
            .parse::<i64>()
            .expect("could not parse x-ratelimit-reset header to i64");
        let used = ratelimit_used
            .to_str()
            .expect("could not parse x-ratelimit-used header to &str")
            .parse::<u32>()
            .expect("could not parse x-ratelimit-used to i64");

        self.remaining = Some(ChronoDuration::seconds(seconds_to_reset.into()));
        self.used = Some(used);
        self.reset_timestamp =
            Utc::now().checked_add_signed(ChronoDuration::seconds(seconds_to_reset));

        if self.remaining.unwrap().is_zero() {
            self.next_request_timestamp = self.reset_timestamp;
            return;
        }

        let reset_timestamp_seconds = self.reset_timestamp.unwrap().timestamp();
        let next_request_timestamp_seconds = self.next_request_timestamp.unwrap().timestamp();
        let remaining = self.remaining.unwrap().num_seconds();

        let nrt = min_by(
            reset_timestamp_seconds,
            now.timestamp()
                + max_by(
                    0,
                    min_by((seconds_to_reset - remaining) / 2, 10, i64::cmp),
                    i64::cmp,
                ),
            i64::cmp,
        );

        self.next_request_timestamp = Some(Utc.timestamp_opt(nrt, 0).unwrap());
    }

    pub fn call<R, H>(&self, request_function: R, set_header_callback: H, data: Value) -> Response
    where
        R: Fn(Value, HeaderMap) -> Response,
        H: Fn() -> HeaderMap,
    {
        self.delay();

        let headers = set_header_callback();
        let response = request_function(data, headers);

        self.update(response.headers().to_owned());

        return response;
    }
}
