use std::sync::Arc;

use base64::Engine;
use comemo::Track;
use ecow::{eco_format, EcoString};
use reflexo::typst::TypstHtmlDocument;
use typst::diag::SourceResult;
use typst::model::LateLinkResolver;
use typst::syntax::Span;
use typst_html::{HtmlElement, HtmlNode};

use crate::hast::{HastElement, HastElementContent, HastText};

pub mod hast;

/// Encodes an HTML document into a Hast.
pub fn hast(document: &Arc<TypstHtmlDocument>) -> SourceResult<HastElementContent> {
    let link_resolver = LateLinkResolver::new(None, document.introspector().as_ref());
    let link_resolver = link_resolver.track();
    write_element(document.root(), link_resolver)
}

/// Encode an HTML node into the writer.
fn write_node(
    node: &HtmlNode,
    buf: &mut Vec<HastElementContent>,
    link_resolver: comemo::Tracked<LateLinkResolver<'_>>,
) -> SourceResult<()> {
    match node {
        HtmlNode::Tag(_) => {}
        HtmlNode::Text(text, span) => {
            buf.push(write_text(text, *span)?);
        }
        HtmlNode::Element(element) => {
            buf.push(write_element(element, link_resolver)?);
        }
        HtmlNode::Frame(frame) => {
            write_frame(frame, buf, link_resolver);
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
fn write_element(
    element: &HtmlElement,
    link_resolver: comemo::Tracked<LateLinkResolver<'_>>,
) -> SourceResult<HastElementContent> {
    write_element_with_tag(element, &element.tag.resolve(), link_resolver)
}

/// Encode one element into the write.
fn write_element_with_tag(
    element: &HtmlElement,
    tag: &str,
    link_resolver: comemo::Tracked<LateLinkResolver<'_>>,
) -> SourceResult<HastElementContent> {
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
            write_node(c, &mut buf, link_resolver)?;
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
fn write_frame(
    frame: &typst_html::HtmlFrame,
    buf: &mut Vec<HastElementContent>,
    link_resolver: comemo::Tracked<LateLinkResolver<'_>>,
) {
    // FIXME: This string replacement is obviously a hack.
    let svg = typst_svg::svg_in_html(
        &frame.inner,
        frame.text_size,
        frame.id.as_deref(),
        &eco_format!("{}", frame.css.to_inline()),
        &frame.anchors,
        link_resolver,
    )
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
