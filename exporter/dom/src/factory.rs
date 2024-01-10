use web_sys::{wasm_bindgen::JsCast, Element, HtmlTemplateElement};

#[derive(Clone)]
pub struct XmlFactory(HtmlTemplateElement);

impl Default for XmlFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlFactory {
    pub fn new() -> Self {
        Self(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("template")
                .unwrap()
                .dyn_into()
                .unwrap(),
        )
    }

    pub fn create_element(&self, html: &str) -> Element {
        let tmpl = &self.0;
        tmpl.set_inner_html(html);
        tmpl.content().first_element_child().unwrap()
    }
}
