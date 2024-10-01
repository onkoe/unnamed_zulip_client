use organizations::ServerSettingsCache;
use reqwest::{Client as ReqwestClient, RequestBuilder, Url};

use crate::{config::ClientConfig, error::ZulipError};

pub mod config;
pub mod error;
pub mod messages;
pub mod narrow;
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
    pub server_settings_cache: ServerSettingsCache,

    /// the URL to connect to for this server.
    ///
    /// DO NOT USE THIS FIELD MANUALLY - many methods require `&mut self`,
    /// which allows modification. Instead, go for `self.api_url()`!
    __api_url: Url,
    client: ReqwestClient,
}

impl Client {
    #[tracing::instrument]
    pub async fn new(conf: ClientConfig) -> Result<Self, ZulipError> {
        let server_address = conf.server_address.clone();

        let (reqwest_client, api_url) = futures::join! {
            Self::make_reqwest_client(),
            Self::make_api_url(&server_address),
        };

        let server_settings_cache = ServerSettingsCache::new(
            reqwest_client,
            &api_url,
            conf.server_settings_cache_interval.clone(),
        )
        .await?;

        let client = Client {
            conf,
            server_settings_cache,

            __api_url: api_url,
            client: ReqwestClient::new(),
        };

        Ok(client)
    }

    pub fn reqwest_client(&self) -> ReqwestClient {
        self.client.clone()
    }
}

impl Client {
    /// Makes the API URL. For use only during construction.
    ///
    /// This is in associated function form to allow making this during `Self`
    /// construction. ALWAYS use the `api_url` field after construction.
    async fn make_api_url(server_address: &Url) -> Url {
        let addr = server_address.clone();

        tokio::task::spawn_blocking(move || {
            addr.join("/api/v1/")
                .expect("the api part of the addr should always be correct")
        })
        .await
        .expect("the tokio task for modifying a url should never panic")
    }

    /// Makes the API URL (for example, `https://my.url/api/v1/`) from the
    /// given `server_address`, (such as `https://my.url`).
    pub fn api_url(&self) -> Url {
        self.__api_url.clone()
    }

    async fn make_reqwest_client() -> ReqwestClient {
        tokio::task::spawn_blocking(ReqwestClient::new)
        .await
        .expect("there was something wrong with your system configuration. `reqwest` was unable to find the required TLS library, or no system configuration was available.")
    }

    /// Apply authentication to the created `RequestBuilder` using internal
    /// mechanisms.
    ///
    /// Don't change this without thorough testing!
    fn auth(&self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.basic_auth(self.conf.email.clone(), Some(self.conf.api_key.get()))
    }
}
