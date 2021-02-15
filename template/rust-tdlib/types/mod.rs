//! Module provides all TDlib types.
//! For types details you can see [TDlib API Scheme](https://github.com/tdlib/td/blob/master/td/generate/scheme/td_api.tl)
pub(crate) use self::_common::from_json;
pub(crate) use self::_common::{RFunction, RObject, TdType};

#[macro_use]
mod _common;

{% for key, value in file_obj_map %}pub use self::{{key}}::*;
{% endfor %}


{% for key, value in file_obj_map %}mod {{key}};
{% endfor %}
