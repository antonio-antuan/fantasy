{% set trait_name = token.name | to_camel %}
/// {{token.description}}
pub trait TD{{trait_name}}: Debug + RObject {}

/// {{token.description}}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(tag="@type")]
pub enum {{trait_name}} {
  #[doc(hidden)]
  #[default]
  _Default,
{% for subt in sub_tokens(token=token) %}  /// {{subt.description}}
  #[serde(rename = "{{subt.name}}")]
  {% set variant_name = td_update_variant(variant_name=subt.name | to_camel, enum_name=trait_name) %}
  {{subt.name | td_remove_prefix(prefix=trait_name) | to_camel}}({{variant_name}}),
{% endfor %}
}

impl RObject for {{trait_name}} {
  #[doc(hidden)] fn extra(&self) -> Option<&str> {
    match self {
{% for subt in sub_tokens(token=token) %}      {{trait_name}}::{{subt.name | td_remove_prefix(prefix=trait_name) | to_camel}}(t) => t.extra(),
{% endfor %}
      _ => None,
    }
  }
#[doc(hidden)] fn client_id(&self) -> Option<i32> {
    match self {
{% for subt in sub_tokens(token=token) %}      {{trait_name}}::{{subt.name | td_remove_prefix(prefix=trait_name) | to_camel}}(t) => t.client_id(),
{% endfor %}
      _ => None,
    }
  }
}

impl {{trait_name}} {
  pub fn from_json<S: AsRef<str>>(json: S) -> Result<Self> { Ok(serde_json::from_str(json.as_ref())?) }
  #[doc(hidden)] pub fn _is_default(&self) -> bool { matches!(self, {{trait_name}}::_Default) }
}

impl AsRef<{{trait_name}}> for {{trait_name}} {
  fn as_ref(&self) -> &{{trait_name}} { self }
}
