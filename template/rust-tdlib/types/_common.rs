use std::{
  fmt::{Debug, Display},
  str::FromStr
};

use serde::de::{Deserialize, Deserializer, Error as SerdeError};

use crate::{
  errors::*,
  types::*
};
use serde::{de, Serialize, Serializer};

#[allow(dead_code)]
pub fn from_json<'a, T>(json: &'a str) -> RTDResult<T> where T: serde::de::Deserialize<'a>, {
  Ok(serde_json::from_str(json)?)
}

/// All tdlib type abstract class defined the same behavior
pub trait RObject: Debug {
  #[doc(hidden)]
  fn extra(&self) -> Option<&str>;
  fn client_id(&self) -> Option<i32>;
}

pub trait RFunction: Debug + RObject + Serialize {
  fn to_json(&self) -> RTDResult<String> {
      Ok(serde_json::to_string(self)?)
  }
}


impl<'a, RObj: RObject> RObject for &'a RObj {
  fn extra(&self) -> Option<&str> { (*self).extra() }
  fn client_id(&self) -> Option<i32> { (*self).client_id() }
}

impl<'a, RObj: RObject> RObject for &'a mut RObj {
  fn extra(&self) -> Option<&str> { (**self).extra() }
  fn client_id(&self) -> Option<i32> { (**self).client_id() }
}


impl<'a, Fnc: RFunction> RFunction for &'a Fnc {}
impl<'a, Fnc: RFunction> RFunction for &'a mut Fnc {}

{% for token in tokens %}{% if token.type_ == 'Trait' %}
impl<'a, {{token.name | upper}}: TD{{token.name | to_camel}}> TD{{token.name | to_camel}} for &'a {{token.name | upper}} {}
impl<'a, {{token.name | upper}}: TD{{token.name | to_camel}}> TD{{token.name | to_camel}} for &'a mut {{token.name | upper}} {}
{% endif %}{% endfor %}

#[derive(Debug, Clone)]
pub(crate) enum TdType {
{% for token in tokens %}{% if token.is_return_type %} {{token.name | to_camel }}({{token.name | to_camel}}),
{% endif %}{% endfor %}
}
impl<'de> Deserialize<'de> for TdType {
fn deserialize<D>(deserializer: D) -> Result<TdType, D::Error> where D: Deserializer<'de> {
    use serde::de::Error;
    let rtd_trait_value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    let rtd_trait_map = match rtd_trait_value.as_object() {
        Some(map) => map,
        None => return Err(D::Error::unknown_field(stringify!( TdType ), &[stringify!( "{} is not the correct type" , TdType )]))
    };

    let rtd_trait_type = match rtd_trait_map.get("@type") {
        Some(t) => match t.as_str() {
            Some(s) => s,
            None => return Err(D::Error::unknown_field(stringify!( "{} -> @type" , $field ), &[stringify!( "{} -> @type is not the correct type" , TdType )]))
        },
        None => return Err(D::Error::custom("@type is empty"))
    };
    if let Some(t) = deserialize_traits(rtd_trait_type, rtd_trait_value.clone()).map_err(|err|D::Error::custom(format!("can't deserialize for {} with error: {}", rtd_trait_type, err)))? {
      return Ok(t)
    };
    if let Some(t) = deserialize_direct_types(rtd_trait_type, rtd_trait_value.clone()).map_err(|err|D::Error::custom(format!("can't deserialize for {} with error: {}", rtd_trait_type, err)))? {
      return Ok(t)
    }
    Err(D::Error::custom(format!("got {} @type with unavailable variant", rtd_trait_type)))
 }
}

fn deserialize_traits(rtd_trait_type: &str, rtd_trait_value: serde_json::Value) -> Result<Option<TdType>, serde_json::Error> {
  {% for token in tokens %}{% if token.is_return_type and token.type_ == "Trait" %}
    if let Some(td_type) = deserialize_{{token.name | to_snake}}(rtd_trait_type, rtd_trait_value.clone())? {
      return Ok(Some(td_type))
    };
    {% endif %}
    {% endfor %}
    Ok(None)
}

fn deserialize_direct_types(rtd_trait_type: &str, rtd_trait_value: serde_json::Value) -> Result<Option<TdType>, serde_json::Error> {
  Ok(match rtd_trait_type { {% for token in tokens %}{% if token.is_return_type and token.type_ != "Trait" %}
  "{{token.name}}" => Some(TdType::{{token.name | to_camel}}(
          serde_json::from_value(
              rtd_trait_value
          )?
      )),{% endif %}{% endfor %}
      _ => None
    })
}

{% for token in tokens %}{% if token.is_return_type and token.type_ == "Trait" %}
const {{token.name | to_upper}}_MEMBERS: &[&str] = &[{% for subt in sub_tokens(token = token) %}"{{subt.name}}", {% endfor %}];

  fn deserialize_{{ token.name | to_snake }}(rtd_trait_type: &str, rtd_trait_value: serde_json::Value) -> Result<Option<TdType>, serde_json::Error> {
    Ok(match {{token.name | to_upper}}_MEMBERS.contains(&rtd_trait_type) {
      true => Some(TdType::{{token.name | to_camel}}(serde_json::from_value(rtd_trait_value)?)),
      false => None
    })
}
{% endif %}
{% endfor %}

impl TdType {
    pub fn extra(&self) -> Option<&str> {
      match self {
{% for token in tokens %}{% if token.is_return_type %}
          TdType::{{token.name | to_camel}}(value) => value.extra(),
{% endif %}{% endfor %}
      }
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

pub(super) fn serialize_enum_default<S>(s: S) -> Result<S::Ok, S::Error> where S: Serializer  {
    Ok(S::serialize(None).ok())
}

pub(super) fn vec_of_i64_from_str<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Vec::<String>::deserialize(deserializer)?;
    let mut r = Vec::new();
    for v in s {
        match v.parse::<i64>() {
            Ok(v) => {r.push(v)}
            Err(e) => {return Err(D::Error::custom(format!("can't deserialize to i64: {}", e)))}
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use crate::types::_common::deserialize_update;
    use crate::types::{from_json, AuthorizationState, TdType, Update};

    #[test]
    fn test_deserialize_enums() {
        match deserialize_update(
            "updateAuthorizationState", serde_json::from_str::<serde_json::Value>(r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateWaitTdlibParameters"}}"#).unwrap(),
        ) {
            Ok(v) => {match v {
                Some(v) => {
                    match v {
                        TdType::Update(_) => {},

                        _ => {panic!("serialization failed")},
                    }
                },
                None => panic!("serialization failed")
            }}
            Err(e) => {
                panic!("{}", e)
            }
        };

        match from_json::<TdType>(
            r#"{"@type":"updateAuthorizationState","authorization_state":{"@type":"authorizationStateWaitTdlibParameters"}}"#,
        ) {
            Ok(t) => match t {
                TdType::Update(Update::AuthorizationState(state)) => {
                    match state.authorization_state() {
                        AuthorizationState::WaitTdlibParameters(_) => {}
                        _ => {
                            panic!("invalid serialized data")
                        }
                    }
                }
                _ => panic!("from_json failed: {:?}", t),
            },
            Err(e) => {
                panic!("{}", e)
            }
        };
    }
}
