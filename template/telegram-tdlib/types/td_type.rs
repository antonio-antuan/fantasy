{% if first_write %}
use crate::types::*;
use crate::errors::*;
use uuid::Uuid;
{% endif %}

{% if token.type_ == "Trait" %}
{% if first_write %}
use std::fmt::Debug;
use serde::de::{Deserialize, Deserializer};
{% endif %}

{% include "telegram-tdlib/types/td_type_trait.rs" %}
{% else %}
{% include "telegram-tdlib/types/td_type_struct.rs" %}
{% endif %}