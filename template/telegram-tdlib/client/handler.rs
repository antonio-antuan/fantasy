
use super::api::aevent::EventApi;
use super::listener::Lout;
use super::errors::TGError;
use super::observer;
use super::tip;

use crate::types as rtd_types;

pub struct Handler<'a> {
  api: &'a EventApi,
  lout: &'a Lout,
  warn_unregister_listener: &'a bool,
}

impl<'a> Handler<'a> {
  pub(crate) fn new(api: &'a EventApi, lout: &'a Lout, warn_unregister_listener: &'a bool) -> Self {
    Self {
      api,
      lout,
      warn_unregister_listener,
    }
  }

  pub fn handle(&self, json: &'a String) {
    if let Some(ev) = self.lout.receive() {
      if let Err(e) = ev((self.api, json)) {
        if let Some(ev) = self.lout.exception() { ev((self.api, &e)); }
      }
    }
    match rtd_types::from_json::<rtd_types::TdType>(json) {
      Ok(t) => {
        match self.lout.handle_type(self.api, &t) {
          Ok(true) => return,
          Ok(false) => {
            if *self.warn_unregister_listener {
              warn!("{}", tip::un_register_listener(stringify!(t)));
            }
          }
          Err(_err) => {
            if let Some(ev) = self.lout.exception() {
              ev((self.api, &TGError::new("EVENT_HANDLER_ERROR")));
            }
          }
        }

        observer::notify(t);
      }
      Err(e) => {
        error!("{}", tip::data_fail_with_json(json));
        error!("{:?}", e);
        if let Some(ev) = self.lout.exception() { ev((self.api, &TGError::new("DESERIALIZE_JSON_FAIL"))); }
      }
    }
  }
}
