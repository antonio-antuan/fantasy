use std::sync::RwLock;
use std::collections::HashMap;
use futures::channel::oneshot;
use crate::types::{RObject, TdType};

lazy_static! {
  pub static ref OBSERVER: Observer = Observer::new();
}

pub struct Observer {
  channels: RwLock<HashMap<String, oneshot::Sender<TdType>>>,
}

impl Observer {
  fn new() -> Self {
    Self {
      channels: RwLock::new(HashMap::new())
    }
  }

  pub fn notify(&self, payload: TdType) -> Option<TdType> {
    let extra = match &payload {
{% for token in tokens %}{% if token.is_return_type %}
      TdType::{{token.name | to_camel}}(value) => value.extra(),
{% endif %}{% endfor %}
      _ => {None}
    };
    match extra {
      None => Some(payload),
      Some(extra) => {
          let mut map = self.channels.write().unwrap();
          match map.remove(&extra) {
              None => {Some(payload)}
              Some(sender) => {
                sender.send(payload).unwrap();
                None
              }
          }
      }
    }
  }

  pub fn subscribe(&self, extra: &String) -> oneshot::Receiver<TdType> {
    let (sender, receiver) = oneshot::channel::<TdType>();
    match self.channels.write() {
      Ok(mut map) => {
        map.insert(extra.to_string(), sender);
      }
      _ => {}
    };
    receiver
  }

  pub fn unsubscribe(&self, extra: &String) {
    match self.channels.write() {
      Ok(mut map) => {
        map.remove(extra);
      }
      _ => {}
    };
  }
}
