use super::{
  observer::OBSERVER,
  tdlib_client::TdLibClient,
};
use crate::{
  errors::{RTDError, RTDResult},
  types::*,
};
use super::Client;

const CLOSED_RECEIVER_ERROR: RTDError = RTDError::Internal("receiver already closed");
const INVALID_RESPONSE_ERROR: RTDError = RTDError::Internal("receive invalid response");
const NO_EXTRA: RTDError =
  RTDError::Internal("invalid tdlib response type, not have `extra` field");

impl<R> Client<R>
where
    R: TdLibClient + Clone,
{
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
      Ok(v) => match serde_json::from_value::<{{token.blood | to_camel}}>(v) {
        Ok(v) => { Ok(v) }
        Err(e) => {
          log::error!("response serialization error: {:?}", e);
          Err(INVALID_RESPONSE_ERROR)
        }
      }
    }
  }
  {% endif %}{% endfor %}
}
