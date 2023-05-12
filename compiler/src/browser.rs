use typst::syntax::SourceId;
use typst_ts_core::font::FontResolverImpl;

use wasm_bindgen::prelude::*;

use crate::font::web::BrowserFontSearcher;

/// A world that provides access to the browser.
pub struct TypstBrowserWorld {
    pub font_resolver: FontResolverImpl,
    pub main: SourceId,
}

impl TypstBrowserWorld {
    // todo: better parameter type
    pub async fn new(searcher: BrowserFontSearcher) -> Result<Self, JsValue> {
        // Hook up the lang items.
        // todo: bad upstream changes
        // 24MB
        // let library = typst_library::build();
        // typst::eval::set_lang_items(library.items.clone());

        // 13MB
        // let dummy_library = typst::eval::LangItems {
        //     layout: |_world, _content, _styles| panic!("layout"),
        //     em: |_styles| panic!("em"),
        //     dir: |_styles| panic!("dir"),
        //     space: || panic!("space"),
        //     linebreak: || panic!("linebreak"),
        //     text: |_text| panic!("text"),
        //     text_func: typst_library::text::TextElem::func(),
        //     text_str: |_content| panic!("text_str"),
        //     smart_quote: |_double| panic!("smart_quote"),
        //     parbreak: || panic!("parbreak"),
        //     strong: |_body| panic!("strong"),
        //     emph: |_body| panic!("emph"),
        //     raw: |_text, _tag, _block| panic!("raw"),
        //     raw_languages: || panic!("raw_languages"),
        //     link: |_url| panic!("link"),
        //     reference: |_target, _supplement| panic!("reference"),
        //     bibliography_keys: |_world, _introspector| panic!("bibliography_keys"),
        //     heading: |_level, _title| panic!("heading"),
        //     heading_func: typst_library::meta::HeadingElem::func(),
        //     list_item: |_body| panic!("list_item"),
        //     enum_item: |_number, _body| panic!("enum_item"),
        //     term_item: |_term, _description| panic!("term_item"),
        //     equation: |_body, _block| panic!("equation"),
        //     math_align_point: || panic!("math_align_point"),
        //     math_delimited: |_open, _body, _close| panic!("math_delimited"),
        //     math_attach: |_base, _bottom, _top| panic!("math_attach"),
        //     math_accent: |_base, _accent| panic!("math_accent"),
        //     math_frac: |_num, _denom| panic!("math_frac"),
        //     library_method: |_vm, _dynamic, _method, _args, _span| panic!("library_method"),
        // };
        // typst::eval::set_lang_items(dummy_library);

        Ok(Self {
            // library: Prehashed::new(typst_library::build()),
            font_resolver: searcher.into(),
            main: SourceId::detached(),
        })
    }
}
