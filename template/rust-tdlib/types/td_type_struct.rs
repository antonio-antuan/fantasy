{% set struct_name = token.name | to_camel %}
/// {{token.description}}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct {{struct_name}} {
  #[doc(hidden)]
  #[serde(rename(serialize = "@extra", deserialize = "@extra"))]
  extra: Option<String>,
  #[serde(rename(serialize = "@client_id", deserialize = "@client_id"))]
  client_id: Option<i32>,
  {% for field in token.arguments %}/// {{field.description}}
  {% for macro_ in td_macros(arg=field, token=token) %}{{macro_}} {% endfor %}
  {% if field.sign_name == 'type' %}#[serde(rename(serialize = "type", deserialize = "type"))] {% endif %}
  {% set field_type = td_arg(arg=field, token=token) %}
  {% if field.sign_type == "vector" %}{% for c in field.components %}{% if c.sign_type == "int64" %}#[serde(deserialize_with = "super::_common::vec_of_i64_from_str")]{% endif %}{% endfor %}
  {% elif field.sign_type == "int64" %}#[serde(deserialize_with = "super::_common::number_from_string")]{% endif %}
  {{ serde_attr(arg=field, token=token) }}
  {{field.sign_name | td_safe_field}}: {{ field_type }},{% endfor %}
  {% if token.type_ == 'Function' %}
  #[serde(rename(serialize = "@type"))]
  td_type: String
  {% endif %}
}

impl RObject for {{struct_name}} {
  #[doc(hidden)] fn extra(&self) -> Option<&str> { self.extra.as_deref() }
  #[doc(hidden)] fn client_id(&self) -> Option<i32> { self.client_id }
}
{% if token.blood and token.blood | to_snake != token.name | to_snake %}
{% set blood_token = find_token(token_name=token.blood) %}
{% if blood_token.type_ == 'Trait' %}impl TD{{token.blood | to_camel}} for {{struct_name}} {}{% endif %}
{% endif %}
{% if token.type_ == 'Function' %}impl RFunction for {{struct_name}} {}{% endif %}

impl {{struct_name}} {
  pub fn from_json<S: AsRef<str>>(json: S) -> RTDResult<Self> { Ok(serde_json::from_str(json.as_ref())?) }
  pub fn builder() -> RTD{{struct_name}}Builder {
    let mut inner = {{struct_name}}::default();
    inner.extra = Some(Uuid::new_v4().to_string());
    {% if token.type_ == 'Function' %}
    inner.td_type = "{{token.name}}".to_string();
    {% endif %}
    RTD{{struct_name}}Builder { inner }
  }
{% for field in token.arguments %}{% set field_type = td_arg(arg=field, token=token) %}{% set is_primitive = is_primitive(type_ = field_type) %}
  pub fn {{field.sign_name | td_safe_field}}(&self) -> {% if not is_primitive %}&{% endif %}{{field_type}} { {% if not is_primitive %}&{% endif %}self.{{field.sign_name | td_safe_field}} }
{% endfor %}
}

#[doc(hidden)]
pub struct RTD{{struct_name}}Builder {
  inner: {{struct_name}}
}

impl RTD{{struct_name}}Builder {
  pub fn build(&self) -> {{struct_name}} { self.inner.clone() }
{% for field in token.arguments %}
{% set builder_field_type=td_arg(arg=field, token=token, builder_arg=true) %} {% set sign_name = field.sign_name | td_safe_field %} {% set is_optional = is_optional(type_=td_arg(arg=field, token=token)) %} {% set is_builder_ref = is_builder_ref(type_ = builder_field_type) %}
  pub fn {{sign_name}}{%if is_builder_ref%}<T: AsRef<{% if builder_field_type == 'String' %}str{% else %}{{builder_field_type}}{% endif %}>>{%endif%}(&mut self, {{sign_name}}: {%if is_builder_ref%}T{%else%}{{builder_field_type}}{%endif%}) -> &mut Self {
    self.inner.{{sign_name}} = {% if is_optional %}Some({% endif %}{{sign_name}}{%if is_builder_ref %}.as_ref(){% if builder_field_type == 'String' %}.to_string(){% else %}.clone(){% endif %}{% endif %}{% if is_optional %}){% endif %};
    self
  }
{% endfor %}
}

impl AsRef<{{struct_name}}> for {{struct_name}} {
  fn as_ref(&self) -> &{{struct_name}} { self }
}

impl AsRef<{{struct_name}}> for RTD{{struct_name}}Builder {
  fn as_ref(&self) -> &{{struct_name}} { &self.inner }
}
