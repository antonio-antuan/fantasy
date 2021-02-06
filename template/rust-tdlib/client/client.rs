use crate::{
    client::observer::OBSERVER,
    errors::{RTDError, RTDResult},
    tdjson,
    types::RFunction,
    types::*,
};
use tokio::sync::mpsc;

#[doc(hidden)]
pub trait TdLibClient {
    fn send<Fnc: RFunction>(&self, client_id: tdjson::ClientId, fnc: Fnc) -> RTDResult<()>;
    fn receive(&self, timeout: f64) -> Option<String>;
    fn execute<Fnc: RFunction>(&self, fnc: Fnc) -> RTDResult<Option<String>>;
}

#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct RawApi;

impl Default for RawApi {
    fn default() -> Self {
        Self
    }
}

impl TdLibClient for RawApi {
    fn send<Fnc: RFunction>(&self, client_id: tdjson::ClientId, fnc: Fnc) -> RTDResult<()> {
        let json = fnc.to_json()?;
        tdjson::send(client_id, &json[..]);
        Ok(())
    }

    fn receive(&self, timeout: f64) -> Option<String> {
        tdjson::receive(timeout)
    }

    fn execute<Fnc: RFunction>(&self, fnc: Fnc) -> RTDResult<Option<String>> {
        let json = fnc.to_json()?;
        Ok(tdjson::execute(&json[..]))
    }
}

impl RawApi {
    pub fn new() -> Self {
        Self {}
    }
}

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
    raw_api: S,
    client_id: Option<i32>,
    updates_sender: Option<mpsc::Sender<Box<TdType>>>,
    tdlib_parameters: TdlibParameters,
}

impl<S> Client<S>
where
    S: TdLibClient + Clone,
{
    pub fn tdlib_parameters(&self) -> &TdlibParameters {
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

    pub(crate) fn updates_sender(&self) -> &Option<mpsc::Sender<Box<TdType>>> {
        &self.updates_sender
    }
}

#[derive(Debug)]
pub struct ClientBuilder<R>
where
    R: TdLibClient + Clone,
{
    updates_sender: Option<mpsc::Sender<Box<TdType>>>,
    tdlib_parameters: Option<TdlibParameters>,
    tdjson: R,
}

impl Default for ClientBuilder<RawApi> {
    fn default() -> Self {
        Self {
            updates_sender: None,
            tdlib_parameters: None,
            tdjson: RawApi::new(),
        }
    }
}

impl<R> ClientBuilder<R>
where
    R: TdLibClient + Clone,
{
    pub fn with_updates_sender(mut self, updates_sender: mpsc::Sender<Box<TdType>>) -> Self {
        self.updates_sender = Some(updates_sender);
        self
    }

    /// Base parameters for your TDlib instance.
    pub fn with_tdlib_parameters(mut self, tdlib_parameters: TdlibParameters) -> Self {
        self.tdlib_parameters = Some(tdlib_parameters);
        self
    }

    pub fn with_tdjson<T: TdLibClient + Clone>(self, tdjson: T) -> ClientBuilder<T> {
        ClientBuilder {
            tdjson,
            updates_sender: self.updates_sender,
            tdlib_parameters: self.tdlib_parameters,
        }
    }

    pub fn build(self) -> RTDResult<Client<R>> {
        if self.tdlib_parameters.is_none() {
            return Err(RTDError::BadRequest("tdlib_parameters not set"));
        };

        let client = Client::new(
            self.tdjson,
            self.updates_sender,
            self.tdlib_parameters.unwrap(),
        );
        Ok(client)
    }
}

impl Client<RawApi>
{
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
    pub fn new(
        raw_api: R,
        updates_sender: Option<mpsc::Sender<Box<TdType>>>,
        tdlib_parameters: TdlibParameters,
    ) -> Self {
        Self {
            raw_api,
            updates_sender,
            tdlib_parameters,
            client_id: None,
        }
    }

{% for token in tokens %}{% if token.type_ == 'Function' %}
  // {{ token.description }}
  pub async fn {{token.name | to_snake}}<C: AsRef<{{token.name | to_camel}}>>(&self, {{token.name | to_snake}}: C) -> RTDResult<{{token.blood | to_camel}}> {
    let extra = {{token.name | to_snake }}.as_ref().extra()
      .ok_or(RTDError::Internal("invalid tdlib response type, not have `extra` field"))?;
    let signal = OBSERVER.subscribe(&extra);
    self.raw_api.send(self.get_client_id()?, {{token.name | to_snake }}.as_ref())?;
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

// #[cfg(test)]
// mod tests {
//     use crate::client::api::TdLibClient;
//     use crate::client::client::{Client, ConsoleAuthStateHandler};
//     use crate::errors::RTDResult;
//     use crate::types::{
//         Chats, RFunction, RObject, SearchPublicChats, TdlibParameters, UpdateAuthorizationState,
//     };
//     use std::time::Duration;
//     use tokio::sync::mpsc;
//     use tokio::time::timeout;
//
//     #[derive(Clone)]
//     struct MockedRawApi {
//         to_receive: Option<String>,
//     }
//
//     impl MockedRawApi {
//         pub fn set_to_receive(&mut self, value: String) {
//             trace!("delayed to receive: {}", value);
//             self.to_receive = Some(value);
//         }
//
//         pub fn new() -> Self {
//             Self { to_receive: None }
//         }
//     }
//
//     impl TdLibClient for MockedRawApi {
//         fn send<Fnc: RFunction>(&self, _fnc: Fnc) -> RTDResult<()> {
//             Ok(())
//         }
//
//         fn receive(&self, timeout: f64) -> Option<String> {
//             std::thread::sleep(Duration::from_secs(timeout as u64));
//             if self.to_receive.is_none() {
//                 panic!("value to receive not set");
//             }
//             self.to_receive.clone()
//         }
//
//         fn execute<Fnc: RFunction>(&self, _fnc: Fnc) -> RTDResult<Option<String>> {
//             unimplemented!()
//         }
//     }
//
//     #[tokio::test]
//     async fn test_request_flow() {
//         // here we just test request-response flow with SearchPublicChats request
//         env_logger::init();
//         let mut mocked_raw_api = MockedRawApi::new();
//
//         let search_req = SearchPublicChats::builder().build();
//
//         let chats: serde_json::Value = serde_json::from_str(
//             Chats::builder()
//                 .chat_ids(vec![1, 2, 3])
//                 .build()
//                 .to_json()
//                 .unwrap()
//                 .as_str(),
//         )
//         .unwrap();
//         let mut chats_object = chats.as_object().unwrap().clone();
//         chats_object.insert(
//             "@extra".to_string(),
//             serde_json::Value::String(search_req.extra().unwrap()),
//         );
//         let to_receive = serde_json::to_string(&chats_object).unwrap();
//         mocked_raw_api.set_to_receive(to_receive);
//
//         let client = Client::new(
//             mocked_raw_api.clone(),
//             ConsoleAuthStateHandler::default(),
//             TdlibParameters::builder().build(),
//             None,
//             2.0,
//         );
//
//         let (sx, _rx) = mpsc::channel::<UpdateAuthorizationState>(10);
//         let _updates_handle = client.init_updates_task(sx);
//         trace!("chats objects: {:?}", chats_object);
//         match timeout(
//             Duration::from_secs(10),
//             client.api().search_public_chats(search_req),
//         )
//         .await
//         {
//             Err(_) => panic!("did not receive response within 1 s"),
//             Ok(Err(e)) => panic!("{}", e),
//             Ok(Ok(result)) => assert_eq!(result.chat_ids(), &vec![1, 2, 3]),
//         }
//     }
// }
