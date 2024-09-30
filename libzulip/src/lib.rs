use reqwest::{Client as ReqwestClient, Url};

use crate::config::ClientConfig;

pub mod config;
pub mod error;
pub mod messages;
pub mod organizations;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// The feature level this crate is currently compatible with.
///
/// Features introduced at or before this number are implemented.
pub const FEATURE_LEVEL: u32 = 0;

/// A client that connects with Zulip.
#[derive(Debug)]
pub struct Client {
    pub conf: ClientConfig,

    client: ReqwestClient,
}

impl Client {
    pub fn new(conf: ClientConfig) -> Self {
        Client {
            conf,
            client: ReqwestClient::new(),
        }
    }

    pub fn reqwest_client(&self) -> ReqwestClient {
        self.client.clone()
    }

    fn api_url(&self) -> Url {
        self.conf
            .server_address
            .join("/api/v1/")
            .expect("the api part of the addr should always be correct")
    }
}
