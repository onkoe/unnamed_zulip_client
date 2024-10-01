use reqwest::Url;
use tempfile::NamedTempFile;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use libzulip::{
    build_info,
    config::{ApiKey, ClientConfig, MessagesConfig, UserAgent},
    messages::{
        edit_message::EditedMessage,
        emoji_reaction::EmojiSelector,
        send::{ChannelMessageTarget, Message},
    },
    Client,
};

#[tokio::main]
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
    let client = Client::new(ClientConfig {
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

    // make a uuid to check both ends
    let uuid = Uuid::new_v4();
    tracing::info!("uuid is {uuid}!");

    // ok now run things
    send_message(&client, &uuid, "hello world!".into()).await;
    file_upload(&client, &uuid).await;
    edit_message(&client, &uuid).await;
    delete_message(&client, &uuid).await;
    add_emoji_reaction(&client, &uuid).await;
    remove_emoji_reaction(&client, &uuid).await;
    fetch_message(&client, &uuid).await;
}

#[tracing::instrument(skip_all)]
async fn send_message(client: &Client, uuid: &Uuid, msg: String) -> u64 {
    // try sending a message!
    let resp = client
        .send_message(&Message::Channel {
            to: ChannelMessageTarget::Name("general".into()),
            content: format!("{msg} (`{uuid}`)"),
            topic: "greetings".into(),
            queue_id: "".into(),
            local_id: "".into(),
        })
        .await
        .unwrap();

    tracing::info!("all done! :D");
    resp.id
}

#[tracing::instrument(skip_all)]
async fn file_upload(client: &Client, uuid: &Uuid) {
    // make a file and write stuff to it
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), format!("uploaded file {uuid}"))
        .await
        .unwrap();

    // attempt to upload it
    let up_resp = client.upload_file(temp_file.path()).await.unwrap();
    tracing::debug!("{up_resp:#?}");

    // now download it!
    let down_resp = client.download_file(&up_resp.url).await.unwrap();
    tracing::debug!("{down_resp:#?}");

    // read both files
    let (local, downloaded) = futures::join! {
        tokio::fs::read(temp_file.path()),
        tokio::fs::read(down_resp),
    };

    let (local, downloaded) = { (local.expect("local"), downloaded.expect("downloaded")) };
    assert_eq!(
        local, downloaded,
        "file before and after upload should be equal"
    );

    // put it in the chat
    let _resp = client
        .send_message(&Message::Channel {
            to: ChannelMessageTarget::Name("general".into()),
            content: format!(
                "file for uuid! {uuid}, {}",
                client.api_url().join(&up_resp.url).unwrap()
            ),
            topic: "greetings".into(),
            queue_id: "".into(),
            local_id: "".into(),
        })
        .await
        .unwrap();

    tracing::info!("assertions passed! :D");
}

#[tracing::instrument(skip_all)]
async fn edit_message(client: &Client, uuid: &Uuid) {
    // let's send another message, then edit it
    let msg_id = send_message(client, uuid, "this message should be edited".into()).await;

    let edited_message = EditedMessage {
        message_id: msg_id,
        topic: None,
        send_notification_to_old_thread: Some(true),
        send_notification_to_new_thread: Some(true),
        content: Some(format!("edited baby! {uuid}")),
        stream_id: None,
    };

    client.edit_message(edited_message).await.unwrap();
    tracing::info!("all done! :D");
}

#[tracing::instrument(skip_all)]
async fn delete_message(client: &Client, uuid: &Uuid) {
    tracing::info!("this check might break if you don't have admin perms in this server. so make sure u have them! :D");

    // send message, then we can del it
    let msg_id = client
        .send_message(&Message::Channel {
            to: ChannelMessageTarget::Name("general".into()),
            content: format!("this should be deleted... {uuid}"),
            topic: "greetings".into(),
            queue_id: "".into(),
            local_id: "".into(),
        })
        .await
        .unwrap()
        .id;

    client.delete_message(msg_id).await.unwrap();
    client.delete_message(msg_id).await.unwrap_err(); // we shouldn't be able to delete it twice!
    tracing::info!("assertions passed! :D");
}

#[tracing::instrument(skip_all)]
async fn add_emoji_reaction(client: &Client, uuid: &Uuid) {
    // send a message
    let msg_id = send_message(client, uuid, "this msg should have emoji reactions".into()).await;

    // add a reaction to that message
    client
        .add_emoji_reaction(
            msg_id,
            EmojiSelector {
                emoji_name: String::from("grinning_face_with_smiling_eyes"),
                emoji_code: None,
                reaction_type: None,
            },
        )
        .await
        .unwrap();

    tracing::info!("all done! :D");
}

#[tracing::instrument(skip_all)]
async fn remove_emoji_reaction(client: &Client, uuid: &Uuid) {
    // send a message
    let msg_id = send_message(
        client,
        uuid,
        "this msg should've had its emoji reaction removed!".into(),
    )
    .await;
    let selector = EmojiSelector {
        emoji_name: String::from("heart"),
        emoji_code: None,
        reaction_type: None,
    };

    // add a reaction to that message
    client
        .add_emoji_reaction(msg_id, selector.clone())
        .await
        .unwrap();

    // uhh didn't mean to send a heart and it looks weird. better remove it
    client
        .remove_emoji_reaction(msg_id, selector)
        .await
        .unwrap();
}

#[tracing::instrument(skip_all)]
async fn fetch_message(client: &Client, uuid: &Uuid) {
    // send a message
    const MSG_CONTENT: &str = "`fetch_message`.";
    let msg_id = send_message(client, uuid, MSG_CONTENT.into()).await;

    // add a reaction
    const CRAB_EMOJI: &str = "crab";
    client
        .add_emoji_reaction(msg_id, EmojiSelector::new_from_name(CRAB_EMOJI))
        .await
        .unwrap();

    // grab its info
    let msg = client
        .fetch_single_message(msg_id, false)
        .await
        .unwrap()
        .message;

    // check its contents and reaction
    assert_eq!(msg.content, MSG_CONTENT);
    assert_eq!(
        msg.reactions.unwrap().first().unwrap().emoji_name,
        CRAB_EMOJI
    );

    tracing::info!("assertions passed! :D");
}
