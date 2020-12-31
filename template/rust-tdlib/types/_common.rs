use std::{
  fmt::{Debug, Display},
  str::FromStr
};

use serde::de::{Deserialize, Deserializer};

use crate::{
  errors::*,
  types::*
};
use serde::de;

macro_rules! rtd_enum_deserialize {
  ($type_name:ident, $(($td_name:ident, $enum_item:ident));*;) => {
    // example json
    // {"@type":"authorizationStateWaitEncryptionKey","is_encrypted":false}
    |deserializer: D| -> Result<$type_name, D::Error> {
      let rtd_trait_value: serde_json::Value = Deserialize::deserialize(deserializer)?;
      // the `rtd_trait_value` variable type is &serde_json::Value, tdlib trait will return a object, convert this type to object `&Map<String, Value>`
      let rtd_trait_map = match rtd_trait_value.as_object() {
        Some(map) => map,
        None => return Err(D::Error::unknown_field(stringify!($type_name), &[stringify!("{} is not the correct type", $type_name)])) // &format!("{} is not the correct type", stringify!($field))[..]
      };
      // get `@type` value, detect specific types
      let rtd_trait_type = match rtd_trait_map.get("@type") {
        // the `t` variable type is `serde_json::Value`, convert `t` to str
        Some(t) => match t.as_str() {
          Some(s) => s,
          None => return Err(D::Error::unknown_field(stringify!("{} -> @type", $field), &[stringify!("{} -> @type is not the correct type", $type_name)])) // &format!("{} -> @type is not the correct type", stringify!($field))[..]
        },
        None => return Err(D::Error::missing_field(stringify!("{} -> @type", $field)))
      };

      let obj = match rtd_trait_type {
        $(
          stringify!($td_name) => $type_name::$enum_item(match serde_json::from_value(rtd_trait_value.clone()) {
            Ok(t) => t,
            Err(_e) => return Err(D::Error::unknown_field(stringify!("{} can't deserialize to {}::{}", $td_name, $type_name, $enum_item, _e), &[stringify!("{:?}", _e)]))
          }),
        )*
        _ => return Err(D::Error::missing_field(stringify!($field)))
      };
      Ok(obj)
    }
  }
}

#[allow(dead_code)]
pub fn from_json<'a, T>(json: &'a str) -> RTDResult<T> where T: serde::de::Deserialize<'a>, {
  Ok(serde_json::from_str(json)?)
}

/// All tdlib type abstract class defined the same behavior
pub trait RObject: Debug {
  #[doc(hidden)]
  fn td_name(&self) -> &'static str;
  #[doc(hidden)]
  fn extra(&self) -> Option<String>;
  /// Return td type to json string
  fn to_json(&self) -> RTDResult<String>;
}

pub trait RFunction: Debug + RObject {}


impl<'a, RObj: RObject> RObject for &'a RObj {
  fn td_name(&self) -> &'static str { (*self).td_name() }
  fn to_json(&self) -> RTDResult<String> { (*self).to_json() }
  fn extra(&self) -> Option<String> { (*self).extra() }
}

impl<'a, RObj: RObject> RObject for &'a mut RObj {
  fn td_name(&self) -> &'static str { (**self).td_name() }
  fn to_json(&self) -> RTDResult<String> { (**self).to_json() }
  fn extra(&self) -> Option<String> { (**self).extra() }
}


impl<'a, Fnc: RFunction> RFunction for &'a Fnc {}
impl<'a, Fnc: RFunction> RFunction for &'a mut Fnc {}

{% for token in tokens %}{% if token.type_ == 'Trait' %}
impl<'a, {{token.name | upper}}: TD{{token.name | to_camel}}> TD{{token.name | to_camel}} for &'a {{token.name | upper}} {}
impl<'a, {{token.name | upper}}: TD{{token.name | to_camel}}> TD{{token.name | to_camel}} for &'a mut {{token.name | upper}} {}
{% endif %}{% endfor %}

#[derive(Debug, Clone)]
pub enum TdType {
{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}  {{token.name | to_camel }}({{token.name | to_camel}}),
{% endif %}{% endfor %}
{% for token in tokens %}{% if token.is_return_type %}  {{token.name | to_camel }}({{token.name | to_camel}}),
{% endif %}{% endfor %}
}
impl<'de> Deserialize<'de> for TdType {
fn deserialize<D>(deserializer: D) -> Result<TdType, D::Error> where D: Deserializer<'de> {
    use serde::de::Error;
    rtd_enum_deserialize!(
      TdType,
{% for token in tokens %}{% if token.blood and token.blood == 'Update' %}  ({{token.name }}, {{token.name | to_camel}});
{% endif %}{% endfor %}
{% for token in tokens %}{% if token.is_return_type %}  ({{token.name }}, {{token.name | to_camel}});
{% endif %}{% endfor %}
 )(deserializer)

 }
}



#[cfg(test)]
mod tests {
  use crate::types::{TdType, from_json, UpdateAuthorizationState};

  #[test]
  fn test_deserialize_enum() {
    match from_json::<UpdateAuthorizationState>(r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateWaitTdlibParameters"}}"#) {
      Ok(_) => {},
      Err(e) => {panic!("{}", e)}
    };

    match from_json::<TdType>(r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateWaitTdlibParameters"}}"#) {
      Ok(t) => {
        match t {
          TdType::UpdateAuthorizationState(_) => {},
          _ => panic!("from_json failed: {:?}", t)
        }
      },
      Err(e) => {panic!("{}", e)}
    };
  }
}

pub(super) fn number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
