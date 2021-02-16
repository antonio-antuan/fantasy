use super::{
    observer::OBSERVER,
    tdlib_client::{TdJson, TdLibClient},
};
use crate::{
    errors::{RTDError, RTDResult},
    types::*,
};
use tokio::sync::mpsc;

const CLOSED_RECEIVER_ERROR: RTDError = RTDError::Internal("receiver already closed");
const INVALID_RESPONSE_ERROR: RTDError = RTDError::Internal("receive invalid response");
const NO_EXTRA: RTDError = RTDError::Internal("invalid tdlib response type, not have `extra` field");


/// Represents state of particular client instance.
#[derive(Debug, Clone)]
pub enum ClientState {
    /// Client opened. You can start interaction
    Opened,
    /// Client closed properly. You must reopen it if you want to interact with Telegram
    Closed,
    /// Client closed with error
    Error(String),
}

/// Struct stores all methods which you can call to interact with Telegram, such as:
/// [send_message](Api::send_message), [download_file](Api::download_file), [search_chats](Api::search_chats) and so on.
#[derive(Clone, Debug)]
pub struct Client<S>
where
    S: TdLibClient + Clone,
{
    tdlib_client: S,
    client_id: Option<i32>,
    is_started: bool,
    updates_sender: Option<mpsc::Sender<Update>>,
    tdlib_parameters: TdlibParameters,
}

impl<S> Client<S>
where
    S: TdLibClient + Clone,
{
    pub(crate) fn tdlib_parameters(&self) -> &TdlibParameters {
        &self.tdlib_parameters
    }

    fn get_client_id(&self) -> RTDResult<i32> {
        match self.client_id {
            Some(client_id) => Ok(client_id),
            None => Err(RTDError::BadRequest("client not authorized yet")),
        }
    }

    pub(crate) fn set_client_id(&mut self, client_id: i32) -> RTDResult<()> {
        match self.client_id {
            Some(_) => Err(RTDError::BadRequest("client already authorized")),
            None => {
                self.client_id = Some(client_id);
                self.is_started = true;
                Ok(())
            }
        }
    }

    pub(crate) fn updates_sender(&self) -> &Option<mpsc::Sender<Update>> {
        &self.updates_sender
    }
}

#[derive(Debug)]
pub struct ClientBuilder<R>
where
    R: TdLibClient + Clone,
{
    updates_sender: Option<mpsc::Sender<Update>>,
    tdlib_parameters: Option<TdlibParameters>,
    tdlib_client: R,
}

impl Default for ClientBuilder<TdJson> {
    fn default() -> Self {
        Self {
            updates_sender: None,
            tdlib_parameters: None,
            tdlib_client: TdJson::new(),
        }
    }
}

impl<R> ClientBuilder<R>
where
    R: TdLibClient + Clone,
{
    /// If you want to receive Telegram updates (messages, channels, etc; see `crate::types::Update`),
    /// you must set mpsc::Sender here.
    pub fn with_updates_sender(mut self, updates_sender: mpsc::Sender<Update>) -> Self {
        self.updates_sender = Some(updates_sender);
        self
    }

    /// Base parameters for your TDlib instance.
    pub fn with_tdlib_parameters(mut self, tdlib_parameters: TdlibParameters) -> Self {
        self.tdlib_parameters = Some(tdlib_parameters);
        self
    }

    #[doc(hidden)]
    pub fn with_tdlib_client<T: TdLibClient + Clone>(self, tdlib_client: T) -> ClientBuilder<T> {
        ClientBuilder {
            tdlib_client,
            updates_sender: self.updates_sender,
            tdlib_parameters: self.tdlib_parameters,
        }
    }

    pub fn build(self) -> RTDResult<Client<R>> {
        if self.tdlib_parameters.is_none() {
            return Err(RTDError::BadRequest("tdlib_parameters not set"));
        };

        let client = Client::new(
            self.tdlib_client,
            self.updates_sender,
            self.tdlib_parameters.unwrap(),
        );
        Ok(client)
    }
}

impl Client<TdJson> {
    pub fn builder() -> ClientBuilder<TdJson> {
        ClientBuilder::default()
    }
}
/// TDLib high-level API methods.
/// Methods documentation can be found in https://core.telegram.org/tdlib/docs/td__api_8h.html
impl<R> Client<R>
where
    R: TdLibClient + Clone,
{
    #[doc(hidden)]
    pub fn new(
        tdlib_client: R,
        updates_sender: Option<mpsc::Sender<Update>>,
        tdlib_parameters: TdlibParameters,
    ) -> Self {
        Self {
            tdlib_client,
            updates_sender,
            tdlib_parameters,
            is_started: false,
            client_id: None,
        }
    }

    pub fn set_updates_sender(&mut self, updates_sender: mpsc::Sender<Update>) -> RTDResult<()> {
        match self.is_started {
            true => Err(RTDError::InvalidParameters(
                "can't set updates sender when client already started",
            )),
            false => {
                self.updates_sender = Some(updates_sender);
                Ok(())
            }
        }
    }

    /// Just a shortcut for `crate::client::client::Client::close`, allows you to stop the client.
    pub async fn stop(&self) -> RTDResult<Ok> {
        self.close(Close::builder().build()).await
    }

{% for token in tokens %}{% if token.type_ == 'Function' %}
  // {{ token.description }}
  pub async fn {{token.name | to_snake}}<C: AsRef<{{token.name | to_camel}}>>(&self, {{token.name | to_snake}}: C) -> RTDResult<{{token.blood | to_camel}}> {
    let extra = {{token.name | to_snake }}.as_ref().extra()
      .ok_or(NO_EXTRA)?;
    let signal = OBSERVER.subscribe(&extra);
    self.tdlib_client.send(self.get_client_id()?, {{token.name | to_snake }}.as_ref())?;
    let received = signal.await;
    OBSERVER.unsubscribe(&extra);
    match received {
      Err(_) => {Err(CLOSED_RECEIVER_ERROR)}
      Ok(v) => match v {
        TdType::{{token.blood | to_camel}}(v) => { Ok(v) }
        {% if token.blood != "Error" %}TdType::Error(v) => { Err(RTDError::TdlibError(v.message().clone())) }{% endif %}
        _ => {
          log::error!("invalid response received: {:?}", v);
          Err(INVALID_RESPONSE_ERROR)
        }
      }
    }
  }
  {% endif %}{% endfor %}
}
