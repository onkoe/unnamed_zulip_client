use reqwest::Url;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use libzulip::{
    build_info,
    config::{ApiKey, ClientConfig, MessagesConfig, UserAgent},
    messages::Message,
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
    });

    // make a uuid to check both ends
    let uuid = Uuid::new_v4();
    tracing::info!("uuid is {uuid}!");

    // try sending a message!
    let resp = client
        .send_message(&Message::Channel {
            to: libzulip::messages::ChannelMessageTarget::Name("general".into()),
            content: format!("hello world! {uuid}"),
            topic: "greetings".into(),
            queue_id: "".into(),
            local_id: "".into(),
        })
        .await
        .unwrap();

    dbg!(resp);
}
