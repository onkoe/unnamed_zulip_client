//! Info and settings on a server.

use std::collections::HashMap;

use crate::{error::ZulipError, Client};

impl Client {
    #[tracing::instrument]
    pub async fn server_settings(&self) -> Result<ServerSettings, ZulipError> {
        let url = self.api_url().join("server_settings").unwrap();

        // get em
        let resp = self
            .reqwest_client()
            .get(url)
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("grabbed the server settings!");
        Ok(serde_json::from_str::<ServerSettings>(&resp.text().await?)?)
    }

    #[tracing::instrument]
    pub async fn linkifiers(&self) -> Result<LinkifiersResponse, ZulipError> {
        let url = self.api_url().join("realm/linkifiers").unwrap();

        let resp = self
            .reqwest_client()
            .get(url)
            .basic_auth(self.conf.email.clone(), Some(self.conf.api_key.get()))
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("grabbed the linkifers!");
        Ok(serde_json::from_str::<LinkifiersResponse>(
            &resp.text().await?,
        )?)
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
