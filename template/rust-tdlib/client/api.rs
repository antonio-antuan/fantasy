use super::Client;
use super::tdlib_client::TdLibClient;
use crate::{
  errors::Result,
  types::*,
};


impl<R> Client<R>
where
    R: TdLibClient + Clone,
{
{% for token in tokens %}{% if token.type_ == 'Function' %}
  // {{ token.description }}
  pub async fn {{token.name | to_snake}}<C: AsRef<{{token.name | to_camel}}>>(&self, {{token.name | to_snake}}: C) -> Result<{{token.blood | to_camel}}> {
    self.make_request({{token.name | to_snake}}).await
  }
  {% endif %}{% endfor %}
}
