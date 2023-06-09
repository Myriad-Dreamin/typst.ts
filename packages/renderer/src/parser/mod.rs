pub mod ir;
pub use ir::*;
pub mod artifact;
pub use artifact::*;
use typst_ts_core::error::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Default)]
pub struct JsValueParser {}

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
pub(crate) fn convert_pair(pair: JsValue) -> (JsValue, JsValue) {
    let pair = pair.unchecked_into::<js_sys::Array>();
    (pair.get(0), pair.get(1))
}

impl JsValueParser {
    pub fn parse_bool(&self, field: &str, val: &JsValue) -> ZResult<bool> {
        val.as_bool()
            .ok_or_else(|| error_once!("expected bool", field: field, val: format!("{:?}", val)))
    }

    pub fn parse_f64(&self, field: &str, val: &JsValue) -> ZResult<f64> {
        val.as_f64()
            .ok_or_else(|| error_once!("expected f64", field: field, val: format!("{:?}", val)))
    }

    pub fn parse_string(&self, field: &str, val: &JsValue) -> ZResult<String> {
        val.as_string()
            .ok_or_else(|| error_once!("expected string", field: field, val: format!("{:?}", val)))
    }

    pub fn parse_as_array(&self, field: &str, val: JsValue) -> ZResult<js_sys::Array> {
        val.clone().dyn_into().map_err(error_once_map!(
            "expected array",
            field: field,
            val: format!("{:?}", val)
        ))
    }

    pub fn parse_as_entries(&self, field: &str, val: JsValue) -> ZResult<js_sys::Array> {
        Ok(js_sys::Object::entries(val.dyn_ref().ok_or_else(|| {
            error_once!(
                "expect entries for object",
                field: field,
                val: format!("{:?}", val)
            )
        })?))
    }

    pub fn parse_tv(&self, field: &str, val: JsValue) -> ZResult<(String, Option<JsValue>)> {
        let mut t = String::default();
        let mut sub: Option<JsValue> = None;

        for (k, v) in self.parse_as_entries(field, val)?.iter().map(convert_pair) {
            let k = self.parse_string(field, &k)?;
            match k.as_str() {
                "t" => {
                    t = self.parse_string(field, &v)?;
                }
                "v" => {
                    sub = Some(v);
                }
                _ => {
                    return Err(error_once!(
                        "unknown kv for",
                        field: field,
                        key: k,
                        val: format!("{:?}", v)
                    ))
                }
            }
        }

        Ok((t, sub))
    }

    pub fn parse_tv_expected(&self, field: &str, val: JsValue) -> ZResult<(String, JsValue)> {
        let (t, sub) = self.parse_tv(field, val.clone())?;

        Ok((
            t,
            sub.ok_or_else(|| {
                error_once!(
                    "expected value for {}",
                    field: field,
                    val: format!("{:?}", val)
                )
            })?,
        ))
    }
}
