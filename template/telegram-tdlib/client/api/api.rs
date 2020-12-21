use std::sync::Arc;

use crate::errors::RTDResult;
use crate::Tdlib;
use crate::types::RFunction;



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
}
