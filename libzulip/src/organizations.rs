//! Info and settings on a server.

use reqwest::{Client as ReqwestClient, Url};
use tokio::sync::RwLock;

use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use std::time::Instant;

use crate::{error::ZulipError, Client};

impl Client {
    /// Grabs the API URL for this
    #[tracing::instrument(skip(self))]
    pub async fn linkifiers(&self) -> Result<LinkifiersResponse, ZulipError> {
        let url = self.api_url().join("realm/linkifiers").unwrap();

        let resp = self
            .auth(self.reqwest_client().get(url))
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("grabbed the linkifers!");
        Ok(serde_json::from_str::<LinkifiersResponse>(
            &resp.text().await?,
        )?)
    }
}

/// A cache of the server settings with a required update time.
#[derive(Debug)]
pub struct ServerSettingsCache {
    /// a `ReqwestClient` to perform updates. note that this is just cloned
    /// from the `crate::Client` :D
    reqwest_client: ReqwestClient,
    /// the url where we send api requests
    api_url: Url,

    /// the max amount of time we'll wait before updating the server settings
    refresh_interval: Arc<RwLock<Duration>>,
    /// the time this cache was last updated
    last_updated: Instant,

    /// the list of server settings
    settings: ServerSettings,
}

impl ServerSettingsCache {
    /// The default time to wait until we refresh the cache (currently 5 minutes).
    pub const DEFAULT_CACHE_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 5);

    pub async fn new(
        reqwest_client: ReqwestClient,
        api_url: &Url,
        refresh_interval: Option<Arc<RwLock<Duration>>>,
    ) -> Result<Self, ZulipError> {
        let settings = Self::server_settings(&reqwest_client, api_url).await?;
        let last_updated = Instant::now();

        // use the default if the user didn't provide one
        let refresh_interval = if let Some(itvl) = refresh_interval {
            itvl
        } else {
            let default_itvl = Self::DEFAULT_CACHE_REFRESH_INTERVAL;
            tracing::trace!("the user didn't provide a refresh interval for the server settings cache. using default of {} seconds.", &default_itvl.as_secs());
            Arc::new(RwLock::new(default_itvl))
        };

        Ok(Self {
            reqwest_client,
            api_url: api_url.clone(),

            refresh_interval,
            last_updated,

            settings,
        })
    }

    /// Grabs the server settings directly from Zulip.
    pub async fn get_without_cache(&mut self) -> Result<ServerSettings, ZulipError> {
        self.update().await?;
        Ok(self.settings.clone())
    }

    /// Grabs the server settings. This value may be cached if it has expired.
    pub async fn get(&mut self) -> Result<ServerSettings, ZulipError> {
        // we'll check if the cache has expired and update if needed
        if Instant::now().duration_since(self.last_updated) > *self.refresh_interval.read().await {
            self.update().await?;
        }

        Ok(self.settings.clone())
    }
}

// private
impl ServerSettingsCache {
    /// Grabs the server settings from the Zulip server at `api_url`.
    async fn server_settings(
        reqwest_client: &ReqwestClient,
        api_url: &Url,
    ) -> Result<ServerSettings, ZulipError> {
        let url = api_url.join("server_settings").unwrap();

        // get em
        let resp = reqwest_client.get(url).send().await?.error_for_status()?;

        tracing::trace!("grabbed the server settings!");
        Ok(serde_json::from_str::<ServerSettings>(&resp.text().await?)?)
    }

    /// Updates the cache unconditionally.
    async fn update(&mut self) -> Result<(), ZulipError> {
        self.settings = Self::server_settings(&self.reqwest_client, &self.api_url).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct ServerSettings {
    pub authentication_methods: HashMap<String, serde_json::Value>,
    pub external_authentication_methods: Vec<ExternalAuthenticationMethod>,
    pub zulip_feature_level: u64,
    pub zulip_version: String,
    pub zulip_merge_base: String,
    pub push_notifications_enabled: bool,
    pub is_incompatible: bool,
    pub email_auth_enabled: bool,
    pub require_email_format_usernames: bool,
    realm_uri: String, // TODO: this is deprecated. will it be removed?
    pub realm_name: String,
    pub realm_icon: String,
    pub realm_description: String,
    pub realm_web_public_access_enabled: bool,
}

impl ServerSettings {
    pub fn realm_url(&self) -> String {
        self.realm_uri.clone()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct ExternalAuthenticationMethod {
    pub name: String,
    pub display_name: String,
    pub display_icon: String,
    pub login_url: String,
    pub signup_url: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct LinkifiersResponse {
    pub result: String,
    pub msg: String,
    pub linkifiers: Linkifiers,
}

pub type Linkifiers = Vec<Linkifier>;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Linkifier {
    pub pattern: String,
    pub url_template: String,
    pub id: u64,
}
