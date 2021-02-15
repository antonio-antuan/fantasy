use super::{
    observer::OBSERVER,
    tdlib_client::{RawApi, TdLibClient},
};
use crate::{
    errors::{RTDError, RTDResult},
    tdjson,
    types::*,
};
use tokio::sync::mpsc;


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

impl Default for ClientBuilder<RawApi> {
    fn default() -> Self {
        Self {
            updates_sender: None,
            tdlib_parameters: None,
            tdlib_client: RawApi::new(),
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

impl Client<RawApi> {
    pub fn builder() -> ClientBuilder<RawApi> {
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
            client_id: None,
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
      .ok_or(RTDError::Internal("invalid tdlib response type, not have `extra` field"))?;
    let signal = OBSERVER.subscribe(&extra);
    self.tdlib_client.send(self.get_client_id()?, {{token.name | to_snake }}.as_ref())?;
    let received = signal.await;
    OBSERVER.unsubscribe(&extra);
    match received {
      Err(_) => {Err(RTDError::Internal("receiver already closed"))}
      Ok(v) => match v {
        TdType::{{token.blood | to_camel}}(v) => { Ok(v) }
        {% if token.blood != "Error" %}TdType::Error(v) => { Err(RTDError::TdlibError(v.message().clone())) }{% endif %}
        _ => {
          error!("invalid response received: {:?}", v);
          Err(RTDError::Internal("receive invalid response"))
        }
      }
    }
  }
  {% endif %}{% endfor %}
}


#[cfg(test)]
mod tests {
    use crate::client::client::Client;
    use crate::client::tdlib_client::TdLibClient;
    use crate::client::worker::Worker;
    use crate::errors::RTDResult;
    use crate::tdjson;
    use crate::client::observer::OBSERVER;
    use crate::types::{Chats, RFunction, RObject, SearchPublicChats, TdlibParameters};
    use std::time::Duration;
    use tokio::time::timeout;

    #[derive(Clone)]
    struct MockedRawApi {
        to_receive: Option<String>,
    }

    impl MockedRawApi {
        pub fn set_to_receive(&mut self, value: String) {
            trace!("delayed to receive: {}", value);
            self.to_receive = Some(value);
        }

        pub fn new() -> Self {
            Self { to_receive: None }
        }
    }

    impl TdLibClient for MockedRawApi {
        fn send<Fnc: RFunction>(&self, _client_id: tdjson::ClientId, fnc: Fnc) -> RTDResult<()> {
            Ok(())
        }

        fn receive(&self, timeout: f64) -> Option<String> {
            self.to_receive.clone()
        }

        fn execute<Fnc: RFunction>(&self, _fnc: Fnc) -> RTDResult<Option<String>> {
            unimplemented!()
        }

        fn new_client(&self) -> tdjson::ClientId {
            1
        }
    }

    #[tokio::test]
    async fn test_request_flow() {
        // here we just test request-response flow with SearchPublicChats request
        env_logger::init();

        let mut mocked_raw_api = MockedRawApi::new();

        let search_req = SearchPublicChats::builder().build();
        let chats = Chats::builder().chat_ids(vec![1, 2, 3]).build();
        let chats: serde_json::Value = serde_json::to_value(chats).unwrap();
        let mut chats_object = chats.as_object().unwrap().clone();
        chats_object.insert(
            "@client_id".to_string(),
            serde_json::Value::Number(1.into())
        );
        chats_object.insert(
            "@extra".to_string(),
            serde_json::Value::String(search_req.extra().unwrap().to_string()),
        );
        chats_object.insert(
            "@type".to_string(),
            serde_json::Value::String("chats".to_string()),
        );
        let to_receive = serde_json::to_string(&chats_object).unwrap();
        mocked_raw_api.set_to_receive(to_receive);
        trace!("chats objects: {:?}", chats_object);

        let mut worker = Worker::builder().with_tdlib_client(mocked_raw_api.clone()).build().unwrap();
        worker.start();

        let client = worker.set_client(Client::builder()
                    .with_tdlib_client(mocked_raw_api.clone())
                    .with_tdlib_parameters(TdlibParameters::builder().build())
                    .build()
                    .unwrap()).await;

        match timeout(
            Duration::from_secs(10),
            client.search_public_chats(search_req),
        )
        .await
        {
            Err(_) => panic!("did not receive response within 1 s"),
            Ok(Err(e)) => panic!("{}", e),
            Ok(Ok(result)) => assert_eq!(result.chat_ids(), &vec![1, 2, 3]),
        }
    }
}
