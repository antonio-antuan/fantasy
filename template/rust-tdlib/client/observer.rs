use std::sync::RwLock;
use std::collections::HashMap;
use futures::channel::oneshot;
use crate::types::{RObject, TdType};

lazy_static! {
  pub(super) static ref OBSERVER: Observer = Observer::new();
}

pub(super) struct Observer {
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
      None => {
          trace!("no extra for payload {:?}", payload);
          Some(payload)
      },
      Some(extra) => {
          let mut map = self.channels.write().unwrap();
          match map.remove(&extra) {
              None => {
                  trace!("no subscribers for {}", extra);
                  Some(payload)
              },
              Some(sender) => {
                  trace!("signal send for {}", extra);
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
              trace!("subscribed for {}", extra);
          }
          _ => {warn!("can't acquire lock for notifier map");}
      };
      receiver
  }

  pub fn unsubscribe(&self, extra: &String) {
      match self.channels.write() {
          Ok(mut map) => {
              trace!("remove {} subscription", &extra);
              map.remove(extra);
          }
          _ => {}
      };
  }
}

