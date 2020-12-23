use std::sync::{Arc, Mutex};

use super::observer::OBSERVER;

use super::api::Api;
use crate::{
  types::from_json,
  Tdlib,
  types::TdType
};
use tokio::{
  sync::mpsc,
  task::JoinHandle
};

#[derive(Clone)]
pub struct Client {
    stop_flag: Arc<Mutex<bool>>,
    api: Api,
    updates_sender: Option<mpsc::Sender<TdType>>,
}

impl Default for Client {
    fn default() -> Self {
        Client::new(Tdlib::new())
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

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn new(tdlib: Tdlib) -> Self {
        let stop_flag = Arc::new(Mutex::new(false));
        Self {
            stop_flag,
            api: Api::new(tdlib),
            updates_sender: None,
        }
    }

    pub fn set_updates_sender(&mut self, updates_sender: mpsc::Sender<TdType>) {
        self.updates_sender = Some(updates_sender)
    }

    pub fn start(&self) -> JoinHandle<()> {
        let stop_flag = self.stop_flag.clone();
        let api = self.api.clone();
        let updates_sender = self.updates_sender.clone();
        tokio::spawn(async move {
            while !*stop_flag.lock().unwrap() {
                if let Some(json) = api.receive(2.0) {
                    match from_json::<TdType>(&json) {
                        Ok(t) => {
                            match OBSERVER.notify(t) {
                                None => {}
                                Some(t) => match &updates_sender {
                                    None => {}
                                    Some(sender) => {sender.send(t).await.unwrap()}
                                }
                            }
                        }
                        Err(e) => {}
                    };
                }
            }
        })
    }
}
