use std::sync::Arc;

use crate::types::*;
use super::errors::*;
use super::api::Api;


/// Telegram client event listener
#[derive(Clone, Default)]
pub struct Listener {
  exception: Option<Arc<dyn Fn((&Api, &TGError)) + Send + Sync + 'static>>,
  receive: Option<Arc<dyn Fn((&Api, &String)) -> TGResult<()> + Send + Sync + 'static>>,
{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}  {{token.name  | to_snake}}: Option<Arc<dyn Fn((&Api, &{{token.name | to_camel}})) -> TGResult<()> + Send + Sync + 'static>>,
// {% if true %}update{% endif %}
{% endif %}{% endfor %}
}


impl Listener {
  pub fn new() -> Self { Listener::default() }

  pub(crate) fn has_receive_listen(&self) -> bool { self.receive.is_some() }

  pub(crate) fn lout(&self) -> Lout { Lout::new(self.clone()) }


  /// when receive data from tdlib
  pub fn on_receive<F>(&mut self, fnc: F) -> &mut Self where F: Fn((&Api, &String)) -> TGResult<()> + Send + Sync + 'static {
    self.receive = Some(Arc::new(fnc));
    self
  }

  /// when telegram client throw exception
  pub fn on_exception<F>(&mut self, fnc: F) -> &mut Self where F: Fn((&Api, &TGError)) + Send + Sync + 'static {
    self.exception = Some(Arc::new(fnc));
    self
  }
{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}
  /// {{token.description}}
  // {% if true %}update{% endif %}
  pub fn on_{{token.name  | to_snake}}<F>(&mut self, fnc: F) -> &mut Self
    where F: Fn((&Api, &{{token.name | to_camel}})) -> TGResult<()> + Send + Sync + 'static {
    self.{{token.name  | to_snake}} = Some(Arc::new(fnc));
    self
  }
{% endif %}{% endfor %}
}


/// Get listener
pub struct Lout {
  listener: Listener,
  supports: Vec<&'static str>
}

impl Lout {
  fn new(listener: Listener) -> Self {
    let supports = vec![{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}
      "{{token.name}}",{% endif %}{% endfor %}
    ];
    Self { listener, supports }
  }

  pub fn is_support<S: AsRef<str>>(&self, name: S) -> bool {
    self.supports.iter()
      .find(|&&item| item == name.as_ref())
      .is_some()
  }

  pub fn handle_type(&self, api: &Api, td_type: &TdType) -> TGResult<bool>  {
    match td_type {
{% for name, td_type in listener %}{% set token = find_token(token_name = td_type) %}
      // {% if true %}listener{% endif %}
      TdType::{{token.name | to_camel}}(value) => match &self.listener.{{name | to_snake}} {
        None => Ok(false),
        Some(f) => f((api, value)).map(|_|true),
    },
{% endfor %}
{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}
    // {% if true %}update{% endif %}
      TdType::{{token.name | to_camel}}(value) => match &self.listener.{{token.name | to_snake}} {
        None => Ok(false),
        Some(f) => f((api, value)).map(|_|true),
    },
{% endif %}{% endfor %}
      _ => Ok(false)
  }
  }

  /// when telegram client throw exception
  pub fn exception(&self) -> &Option<Arc<dyn Fn((&Api, &TGError)) + Send + Sync + 'static>> {
    &self.listener.exception
  }

  /// when receive data from tdlib
  pub fn receive(&self) -> &Option<Arc<dyn Fn((&Api, &String)) -> TGResult<()> + Send + Sync + 'static>> {
    &self.listener.receive
  }

{% for name, td_type in listener %}{% set token = find_token(token_name = td_type) %}
  /// {{token.description}}
  // {% if true %}listener{% endif %}
  pub fn {{name | to_snake}}(&self) -> &Option<Arc<dyn Fn((&Api, &{{token.name | to_camel}})) -> TGResult<()> + Send + Sync + 'static>> {
    &self.listener.{{name | to_snake}}
  }
{% endfor %}


{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}
  /// {{token.description}}
  // {% if true %}update{% endif %}
  pub fn {{token.name  | to_snake}}(&self) -> &Option<Arc<dyn Fn((&Api, &{{token.name | to_camel}})) -> TGResult<()> + Send + Sync + 'static>> {
    &self.listener.{{token.name  | to_snake}}
  }
{% endif %}{% endfor %}
}


