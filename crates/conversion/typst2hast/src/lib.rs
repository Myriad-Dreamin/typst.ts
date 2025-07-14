use std::sync::Arc;

use base64::Engine;
use ecow::{eco_format, EcoString};
use reflexo::typst::TypstHtmlDocument;
use typst::diag::SourceResult;
use typst_html::{HtmlElement, HtmlNode};
use typst::layout::Frame;
use typst::syntax::Span;

use crate::hast::{HastElement, HastElementContent, HastText};

pub mod hast;

/// Encodes an HTML document into a Hast.
pub fn hast(document: &Arc<TypstHtmlDocument>) -> SourceResult<HastElementContent> {
    write_element(&document.root)
}

/// Encode an HTML node into the writer.
fn write_node(node: &HtmlNode, buf: &mut Vec<HastElementContent>) -> SourceResult<()> {
    match node {
        HtmlNode::Tag(_) => {}
        HtmlNode::Text(text, span) => {
            buf.push(write_text(text, *span)?);
        }
        HtmlNode::Element(element) => {
            buf.push(write_element(element)?);
        }
        HtmlNode::Frame(frame) => {
            write_frame(&frame.inner, buf);
        }
    }
    Ok(())
}

/// Encode plain text into the writer.
fn write_text(text: &EcoString, _span: Span) -> SourceResult<HastElementContent> {
    Ok(HastElementContent::Text(HastText {
        value: EcoString::from(text),
        // todo: span mapping
    }))
}

/// Encode one element into the write.
fn write_element(element: &HtmlElement) -> SourceResult<HastElementContent> {
    write_element_with_tag(element, &element.tag.resolve())
}

/// Encode one element into the write.
fn write_element_with_tag(element: &HtmlElement, tag: &str) -> SourceResult<HastElementContent> {
    let properties = element
        .attrs
        .0
        .iter()
        .map(|(attr, value)| (attr.resolve().as_str().into(), value.clone()))
        .collect();

    // todo: ignored: tag::is_void(element.tag)

    let mut buf = Vec::new();

    if !element.children.is_empty() {
        for c in &element.children {
            write_node(c, &mut buf)?;
        }
    }

    Ok(HastElementContent::Element(Box::new(HastElement {
        tag_name: EcoString::from(tag),
        properties,
        children: buf,
        data: None,
    })))
}

/// Encode a laid out frame into the writer.
fn write_frame(frame: &Frame, buf: &mut Vec<HastElementContent>) {
    // FIXME: This string replacement is obviously a hack.
    let svg = typst_svg::svg_frame(frame)
        .replace("<svg class", "<svg style=\"overflow: visible;\" class");

    // create a img base64
    let base64_svg = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());

    buf.push(HastElementContent::Element(Box::new(HastElement {
        tag_name: EcoString::from("img"),
        properties: std::collections::BTreeMap::from([
            (EcoString::inline("class"), EcoString::inline("typst-frame")),
            (EcoString::inline("data-typst-doc"), EcoString::inline("1")),
            (
                EcoString::inline("src"),
                EcoString::from(format!("data:image/svg+xml;base64,{base64_svg}")),
            ),
        ]),
        children: vec![],
        data: Some(crate::hast::HastElementData {
            // Provides a cheap hash
            hash: Some(eco_format!(
                "siphash128_13:{:016x}",
                reflexo::hash::hash128(&frame)
            )),
        }),
    })));
}
