use std::sync::Arc;

use crate::{
  Tdlib,
  errors::{RTDResult, RTDError},
  types::RFunction,
  client::observer::OBSERVER,
  types::*
};


#[derive(Debug, Clone)]
pub struct Api {
  tdlib: Arc<Tdlib>,
}

impl Default for Api {
  fn default() -> Self {
    Self { tdlib: Arc::new(Tdlib::new()) }
  }
}


impl Api {
  pub fn new(tdlib: Tdlib) -> Self {
    Self { tdlib: Arc::new(tdlib) }
  }

  pub fn send<Fnc: RFunction>(&self, fnc: Fnc) -> RTDResult<()> {
    let json = fnc.to_json()?;
    self.tdlib.send(&json[..]);
    Ok(())
  }

  pub fn receive(&self, timeout: f64) -> Option<String> {
    self.tdlib.receive(timeout)
  }

  pub fn execute<Fnc: RFunction>(&self, fnc: Fnc) -> RTDResult<Option<String>> {
    let json = fnc.to_json()?;
    Ok(self.tdlib.execute(&json[..]))
  }


{% for token in tokens %}{% if token.type_ == 'Function' %}
  pub async fn {{token.name | to_snake}}<C: AsRef<{{token.name | to_camel}}>>(&self, {{token.name | to_snake}}: C) -> RTDResult<{{token.blood | to_camel}}> {
    let extra = {{token.name | to_snake }}.as_ref().extra()
      .ok_or(RTDError::Internal("invalid tdlib response type, not have `extra` field"))?;
    let signal = OBSERVER.subscribe(&extra);
    self.send({{token.name | to_snake }}.as_ref())?;
    let received = signal.await;
    OBSERVER.unsubscribe(&extra);
    match received {
      Err(_) => {Err(RTDError::Internal("receiver already closed"))}
      Ok(v) => match v {
        TdType::{{token.blood | to_camel}}(v) => { Ok(v) }
        TdType::Error(v) => { Err(RTDError::TdlibError(v.message().clone())) }
        _ => {
          error!("invalid response received: {:?}", v);
          Err(RTDError::Internal("receive invalid response"))
        }

      }
    }
  }
{% endif %}{% endfor %}
}
