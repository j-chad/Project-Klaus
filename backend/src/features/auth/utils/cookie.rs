use crate::config::AuthSettings;
use axum_extra::extract::cookie::{Cookie, SameSite};

pub fn new_session_cookie(config: &AuthSettings, value: &str) -> Cookie<'static> {
    Cookie::build((config.session_cookie_name.clone(), value.to_string()))
        .http_only(true)
        .secure(config.session_cookie_secure)
        .same_site(SameSite::Strict)
        .permanent()
        .path("/")
        .build()
}
