//! Template variable substitution and validation.

use std::collections::HashMap;

use crate::templates::{TemplateError, TemplateResult};

/// Parse a slice of `"key=value"` strings into a map.
pub(crate) fn parse_vars(vars: &[String]) -> TemplateResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for v in vars {
        let (k, val) = v
            .split_once('=')
            .ok_or_else(|| TemplateError::InvalidVar { value: v.clone() })?;
        map.insert(k.to_string(), val.to_string());
    }
    Ok(map)
}

/// Replace all `{{key}}` occurrences in `raw` with values from `vars`.
pub(crate) fn substitute(raw: &str, vars: &HashMap<String, String>) -> String {
    let mut result = raw.to_string();
    for (k, v) in vars {
        let placeholder = format!("{{{{{}}}}}", k);
        result = result.replace(&placeholder, v);
    }
    result
}
