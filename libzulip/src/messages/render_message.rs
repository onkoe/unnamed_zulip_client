// POST https://yourZulipDomain.zulipchat.com/api/v1/messages/render

// only param is `content: String`
// only resp is `rendered: String`, `msg: String`, `result: String`, `code: Option<String>`

use std::collections::HashMap;

use crate::{
    error::{MessageError, ResponseError, ZulipError},
    Client,
};

impl Client {
    /// Asks the server to render the given (markdown) message as HTML, then
    /// returns it as a string if successful.
    pub async fn render_message<S>(&self, content: S) -> Result<String, ZulipError>
    where
        S: AsRef<str> + std::fmt::Debug + Send,
    {
        // make the url
        let url = self.api_url().join("messages/render")?;

        // add our only parameter (`content`)
        let content = content.as_ref();
        let parameters = HashMap::from([("content", content)]);

        // render it
        let resp = self
            .auth(self.reqwest_client().post(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?;

        // parse it
        let parsed_resp = serde_json::from_str::<RenderResponse>(&resp.text().await?)?;

        // twist it
        if let Some(error) = parsed_resp.error {
            return Err(MessageError::RenderMessageFailed {
                content: String::from(content),
                error: error.to_string(),
            }
            .into());
        }

        Ok(parsed_resp.rendered)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RenderResponse {
    #[serde(flatten)]
    error: Option<ResponseError>,
    // code: Option<String>,
    // msg: Option<String>,
    rendered: String,
}
