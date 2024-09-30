use reqwest::Url;
use tracing_subscriber::EnvFilter;

use libzulip::{
    build_info,
    config::{ApiKey, ClientConfig, MessagesConfig, UserAgent},
    Client,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // grab auth stuff from env
    let email = std::env::var("ZULIP_EMAIL").unwrap();
    let api_key = std::env::var("ZULIP_PERSONAL_KEY").unwrap();
    let server_address = Url::try_from("https://libz.zulipchat.com").unwrap(); // change if u want

    // setup logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!(
            "info,{}=trace",
            build_info::PKG_NAME
        )))
        .init();

    // make the client
    let mut client = Client::new(ClientConfig {
        user_agent: UserAgent::new("client_name", "version"),
        api_key: ApiKey::new(api_key),
        email,
        server_address,
        messages: MessagesConfig {
            read_by_sender: true,
        },
        server_settings_cache_interval: None,
    })
    .await
    .unwrap();

    // grab the settings!
    let resp = client.server_settings_cache.get().await.unwrap();
    dbg!(resp);

    // and linkifiers...
    let resp_linkifiers = client.linkifiers().await.unwrap();
    dbg!(resp_linkifiers);
}
