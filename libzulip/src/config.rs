use std::{sync::Arc, time::Duration};

use reqwest::Url;
use tokio::sync::RwLock;

use crate::build_info;

#[derive(Clone, Debug)]
pub struct ClientConfig {
    // general stuff
    pub user_agent: UserAgent,
    pub email: String,
    pub api_key: ApiKey,
    pub server_address: Url,

    /// when the cache hasn't been updated for >= this duration, it'll be refreshed
    pub server_settings_cache_interval: Option<Arc<RwLock<Duration>>>,

    // ok now all the little configs for modules
    pub messages: MessagesConfig,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct UserAgent {
    s: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ApiKey {
    key: String,
}

impl ApiKey {
    // TODO: get from sso/etc.

    pub fn new<S: AsRef<str>>(key: S) -> Self {
        Self {
            key: key.as_ref().into(),
        }
    }

    pub fn get(&self) -> String {
        self.key.clone()
    }

    pub fn set<S: AsRef<str>>(&mut self, key: S) {
        self.key = key.as_ref().to_string()
    }
}

impl UserAgent {
    /// Creates a new `UserAgent`.
    pub fn new<S: AsRef<str>>(client_name: S, version: S) -> Self {
        let (client_name, version) = (client_name.as_ref(), version.as_ref());

        let us = format!("{}/{}", build_info::PKG_NAME, build_info::PKG_VERSION);
        let them = format!("{client_name}/{version}");

        UserAgent {
            s: format!("{them}, {us} (Rust)"),
        }
    }

    /// Returns the internal user agent string.
    pub fn get(&mut self) -> String {
        self.s.clone()
    }
}

//
// module configs
//

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MessagesConfig {
    pub read_by_sender: bool,
}
