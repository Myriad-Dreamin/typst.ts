use std::sync::Arc;

use js_sys::Uint8Array;
use reflexo_typst::font::cache::FontInfoCache;
use reflexo_typst::font::memory::MemoryFontSearcher;
use reflexo_typst::font::{BufferFontLoader, FontResolverImpl, FontSlot};
use reflexo_typst::package::registry::{JsRegistry, ProxyContext};
use reflexo_typst::vfs::browser::ProxyAccessModel;
use reflexo_typst::{error::prelude::*, Bytes as TypstBytes};
use typst::text::FontInfo;
use wasm_bindgen::prelude::*;

use crate::TypstCompiler;

#[wasm_bindgen]
pub struct TypstCompilerBuilder {
    access_model: Option<ProxyAccessModel>,
    package_registry: Option<JsRegistry>,
    fb: TypstFontResolverBuilder,
}

#[wasm_bindgen]
impl TypstCompilerBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstCompilerBuilder> {
        console_error_panic_hook::set_once();
        let mut res = Self {
            access_model: None,
            package_registry: None,
            fb: TypstFontResolverBuilder::default(),
        };
        res.set_dummy_access_model()?;
        Ok(res)
    }

    pub fn set_dummy_access_model(&mut self) -> Result<()> {
        self.access_model = Some(ProxyAccessModel {
            context: wasm_bindgen::JsValue::UNDEFINED,
            mtime_fn: js_sys::Function::new_no_args("return 0"),
            is_file_fn: js_sys::Function::new_no_args("return true"),
            real_path_fn: js_sys::Function::new_with_args("path", "return path"),
            read_all_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy AccessModel, please initialize compiler with withAccessModel()')",
            ),
        });
        self.package_registry = Some(JsRegistry {
            context: ProxyContext::new(wasm_bindgen::JsValue::UNDEFINED),
            real_resolve_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy Registry, please initialize compiler with withPackageRegistry()')",
            ),
        });
        Ok(())
    }

    pub async fn set_access_model(
        &mut self,
        context: JsValue,
        mtime_fn: js_sys::Function,
        is_file_fn: js_sys::Function,
        real_path_fn: js_sys::Function,
        read_all_fn: js_sys::Function,
    ) -> Result<()> {
        self.access_model = Some(ProxyAccessModel {
            context,
            mtime_fn,
            is_file_fn,
            real_path_fn,
            read_all_fn,
        });

        Ok(())
    }

    pub async fn set_package_registry(
        &mut self,
        context: JsValue,
        real_resolve_fn: js_sys::Function,
    ) -> Result<()> {
        self.package_registry = Some(JsRegistry {
            context: ProxyContext::new(context),
            real_resolve_fn,
        });

        Ok(())
    }

    // 400 KB
    pub async fn add_raw_font(&mut self, data: Uint8Array) -> Result<(), JsValue> {
        self.fb.add_raw_font(data)?;
        Ok(())
    }

    // 100 KB
    pub async fn add_lazy_font(
        &mut self,
        font: JsValue,
        blob: js_sys::Function,
    ) -> Result<(), JsValue> {
        self.fb.add_lazy_font(font, blob)?;
        Ok(())
    }

    pub async fn build(self) -> Result<TypstCompiler, JsValue> {
        let access_model = self
            .access_model
            .ok_or_else(|| "TypstCompilerBuilder::build: access_model is not set".to_string())?;
        let registry = self.package_registry.ok_or_else(|| {
            "TypstCompilerBuilder::build: package_registry is not set".to_string()
        })?;

        let searcher = self.fb;
        #[cfg(feature = "fonts")]
        let mut searcher = searcher;
        #[cfg(feature = "fonts")]
        searcher.add_embedded();

        TypstCompiler::new(access_model, registry, searcher.base.build())
    }
}

#[derive(Default)]
#[wasm_bindgen]
pub struct TypstFontResolverBuilder {
    /// The base font searcher.
    base: MemoryFontSearcher,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl TypstFontResolverBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstFontResolverBuilder> {
        console_error_panic_hook::set_once();
        Ok(Self {
            base: MemoryFontSearcher::new(),
        })
    }

    pub fn get_font_info(&mut self, buffer: Uint8Array) -> Result<JsValue, JsValue> {
        Ok(crate::get_font_info(buffer))
    }

    /// Adds font data to the searcher.
    pub fn add_raw_font(&mut self, buffer: Uint8Array) -> Result<(), JsValue> {
        let buffer = TypstBytes::new(buffer.to_vec());
        for (i, info) in FontInfo::iter(buffer.as_slice()).enumerate() {
            let buffer = buffer.clone();
            self.base.fonts.push((
                info,
                FontSlot::new(BufferFontLoader {
                    buffer: Some(buffer),
                    index: i as u32,
                }),
            ))
        }
        Ok(())
    }

    // todo: move me to upstream
    /// Adds callback that loads font data lazily to the searcher.
    /// `get_font_info` can be used to get the font info.
    pub fn add_lazy_font(&mut self, font: JsValue, blob: js_sys::Function) -> Result<(), JsValue> {
        let arr: FontInfoCache = serde_wasm_bindgen::from_value(font.clone())?;

        for (index, info) in arr.info.into_iter().enumerate() {
            self.base.fonts.push((
                // todo: unneeded clone
                info.clone(),
                FontSlot::new(JsFontLoader::new(
                    info,
                    font.clone(),
                    blob.clone(),
                    index as u32,
                )),
            ))
        }
        Ok(())
    }

    #[cfg(feature = "fonts")]
    fn add_embedded(&mut self) {
        for font_data in typst_assets::fonts() {
            let buffer = Bytes::new(font_data);

            self.base.fonts.extend(
                Font::iter(buffer)
                    .map(|font| (font.info().clone(), FontSlot::new_loaded(Some(font)))),
            );
        }
    }

    pub async fn build(self) -> Result<TypstFontResolver, JsValue> {
        Ok(TypstFontResolver {
            fonts: Arc::new(self.base.build()),
        })
    }
}

#[wasm_bindgen]
pub struct TypstFontResolver {
    pub(crate) fonts: Arc<FontResolverImpl>,
}

/// A web font loader.
#[derive(Debug)]
pub struct JsFontLoader {
    /// The font info.
    pub info: FontInfo,
    /// The context of the font.
    pub context: JsValue,
    /// The blob loader.
    pub blob: js_sys::Function,
    /// The index in a font file.
    pub index: u32,
}

impl JsFontLoader {
    /// Creates a new web font loader.
    pub fn new(info: FontInfo, context: JsValue, blob: js_sys::Function, index: u32) -> Self {
        Self {
            info,
            context,
            blob,
            index,
        }
    }
}

impl reflexo_typst::font::FontLoader for JsFontLoader {
    fn load(&mut self) -> Option<typst::text::Font> {
        let blob = self.blob.call0(&self.context);
        let blob = blob.ok()?;
        let blob = if let Some(data) = blob.dyn_ref::<js_sys::Uint8Array>() {
            TypstBytes::new(data.to_vec())
        } else {
            wasm_bindgen::throw_str(&format!(
                "Font Blob is not a Uint8Array: {:?}, while loading font: {:?}",
                blob, self.info
            ));
        };

        typst::text::Font::new(blob, self.index)
    }
}

/// Safety: `JsFontLoader` is only used in the browser environment, and we
/// cannot share data between workers.
unsafe impl Send for JsFontLoader {}
