use std::sync::{Arc, Mutex};

use crate::Tdlib;
use super::observer;

use super::api::Api;
use super::tip;
use super::listener::Listener;
use super::errors::TGError;
use crate::types::from_json;
use crate::types::TdType;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct Client {
  stop_flag: Arc<Mutex<bool>>,
  listener: Listener,
  api: Api,
}

impl Default for Client {
  fn default() -> Self {
    Client::new(Api::default())
  }
}

impl Client {
  pub fn set_log_verbosity_level<'a>(level: i32) -> Result<(), &'a str> {
    Tdlib::set_log_verbosity_level(level)
  }

  pub fn set_log_max_file_size(size: i64) {
    Tdlib::set_log_max_file_size(size)
  }

  pub fn set_log_file_path(path: Option<&str>) -> bool {
    Tdlib::set_log_file_path(path)
  }

  pub fn new(api: Api) -> Self {
    let stop_flag = Arc::new(Mutex::new(false));
    Self {
      stop_flag,
      api,
      listener: Listener::new(),
    }
  }

  pub fn start_listen_updates(&self) -> JoinHandle<()> {
    let stop_flag = self.stop_flag.clone();
    let api = self.api.clone();
    let lout = self.listener.lout();
    tokio::spawn(async move {
      let is_stop = stop_flag.lock().unwrap();
      while !*is_stop {
        if let Some(json) = api.receive(2.0) {
          if let Some(ev) = lout.receive() {
            if let Err(e) = ev((&api, &json)) {
              if let Some(ev) = lout.exception() { ev((&api, &e)); }
            }
          }
          match from_json::<TdType>(&json) {
            Ok(t) => {
              match lout.handle_type(&api, &t) {
                Ok(true) => return,
                Ok(false) => {}
                Err(_err) => {
                  if let Some(ev) = lout.exception() {
                    ev((&api, &TGError::new("EVENT_HANDLER_ERROR")));
                  }
                }
              }

              observer::notify(t);
            }
            Err(e) => {
              error!("{}", tip::data_fail_with_json(&json));
              error!("{:?}", e);
              if let Some(ev) = lout.exception() { ev((&api, &TGError::new("DESERIALIZE_JSON_FAIL"))); }
            }
          };
        }
      }
    })
  }

  pub fn listener(&mut self) -> &mut Listener {
    &mut self.listener
  }
}
