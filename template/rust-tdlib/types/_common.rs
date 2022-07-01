use std::{
  fmt::{Debug, Display},
  str::FromStr
};

use serde::de::{Deserialize, Deserializer, Error as SerdeError};

use crate::{
  errors::*,
  types::*
};
use serde::{de, Serialize};

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


pub(super) fn number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
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
