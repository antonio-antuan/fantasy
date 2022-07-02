{% if first_write %}use crate::types::*;
use crate::errors::Result;
use uuid::Uuid;
{% endif %}{% if token.type_ == "Trait" %}
{% if first_write %}use std::fmt::Debug;{% endif %}
{% include "rust-tdlib/types/td_type_trait.rs" %}
{% else %}
{% include "rust-tdlib/types/td_type_struct.rs" %}
{% endif %}
