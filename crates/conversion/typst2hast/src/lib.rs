use std::sync::Arc;

use base64::Engine;
use ecow::EcoString;
use reflexo::typst::TypstHtmlDocument;
// use tinymist_world::{CompilerFeat, ExportComputation, WorldComputeGraph};
use typst::diag::SourceResult;
use typst::html::{HtmlElement, HtmlNode};
use typst::layout::Frame;
use typst::syntax::Span;

use crate::hast::{HastElement, HastElementContent, HastText};

pub mod hast;

// pub type ExportStaticHtmlTask = tinymist_task::ExportHtmlTask;
// pub type StaticHtmlExport = tinymist_task::HtmlExport;
// pub type ExportHtmlTask = tinymist_task::ExportHtmlTask;

// pub struct HtmlOutputExport;

// impl<F: CompilerFeat> ExportComputation<F, TypstHtmlDocument> for
// HtmlOutputExport {     type Output = HtmlOutput;
//     type Config = ExportHtmlTask;

//     fn run(
//         _graph: &Arc<WorldComputeGraph<F>>,
//         doc: &Arc<TypstHtmlDocument>,
//         _config: &ExportHtmlTask,
//     ) -> Result<HtmlOutput> {
//         Ok(static_html(doc)?)
//     }
// }

// fn find_tag_child(element: &HtmlElement, tag: HtmlTag) -> Option<usize> {
//     element.children.iter().position(|node| match node {
//         HtmlNode::Element(e) => e.tag == tag,
//         _ => false,
//     })
// }

/// Encodes an HTML document into a Hast.
pub fn hast(document: &Arc<TypstHtmlDocument>) -> SourceResult<HastElementContent> {
    // Ok(HastElementContent::Root(HastRoot {
    //     children: vec![HastElementContent::Text(HastText {
    //         value: EcoString::from("Hello, Typst!"),
    //     })],
    // }))

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
            write_frame(frame, buf);
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
    })));
}
