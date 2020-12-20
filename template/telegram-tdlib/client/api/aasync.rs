use crate::client::observer;
use crate::types::*;
use crate::errors::{RTDResult, RTDError};

use super::api::Api;

use futures::StreamExt;


#[derive(Clone)]
pub struct AsyncApi {
  api: Api,
}

impl AsyncApi {
  pub fn new(api: Api) -> Self {
    Self { api}
  }

  #[doc(hidden)]
  pub fn api(&self) -> &Api {
    &self.api
  }

{% for token in tokens %}{% if token.type_ == 'Function' %}
  pub async fn {{token.name | to_snake}}<C: AsRef<{{token.name | to_camel}}>>(&self, {{token.name | to_snake}}: C) -> RTDResult<{{token.blood | to_camel}}> {
    let extra = {{token.name | to_snake }}.as_ref().extra()
      .ok_or(RTDError::Internal("invalid tdlib response type, not have `extra` field"))?;
    let mut rec = observer::subscribe(&extra);
    self.api.send({{token.name | to_snake }}.as_ref())?;
    let val = rec.next().await;
    observer::unsubscribe(&extra);
    match val {
      Some(TdType::{{token.blood | to_camel}}(v)) => { Ok(v) }
      Some(TdType::Error(v)) => { Err(RTDError::TdlibError(v.message().clone())) }
      _ => { Err(RTDError::Internal("invalid libtd response type, unexpected `extra` field")) }
    }
  }
{% endif %}{% endfor %}

}
