use std::path::Path;

use typst_library::prelude::eco_format;
use wasm_bindgen::JsValue;

use super::{PackageError, PackageSpec, Registry};

pub struct ProxyRegistry {
    pub context: JsValue,
    pub real_resolve_fn: js_sys::Function,
}

impl Registry for ProxyRegistry {
    fn resolve(&self, spec: &PackageSpec) -> Result<std::sync::Arc<Path>, PackageError> {
        let js_spec = js_sys::Object::new();
        js_sys::Reflect::set(&js_spec, &"name".into(), &spec.name.to_string().into()).unwrap();
        js_sys::Reflect::set(
            &js_spec,
            &"namespace".into(),
            &spec.namespace.to_string().into(),
        )
        .unwrap();
        js_sys::Reflect::set(
            &js_spec,
            &"version".into(),
            &spec.version.to_string().into(),
        )
        .unwrap();
        self.real_resolve_fn
            .call1(&self.context, &js_spec)
            .map(|v| Path::new(&v.as_string().unwrap()).into())
            .map_err(|e| PackageError::Other(Some(eco_format!("{:?}", e))))
    }
}
