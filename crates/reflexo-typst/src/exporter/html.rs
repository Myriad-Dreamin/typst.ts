use std::fmt::Write;
use std::sync::{Arc, OnceLock};

use ecow::EcoString;
use reflexo::error::prelude::*;
use reflexo::typst::TypstHtmlDocument;
use tinymist_world::{CompilerFeat, ExportComputation, WorldComputeGraph};
use typst::diag::{bail, At, SourceResult, StrResult};
use typst::foundations::Repr;
use typst::layout::Frame;
use typst::syntax::Span;
use typst_html::{charsets, tag, HtmlElement, HtmlNode, HtmlTag};

pub type ExportStaticHtmlTask = tinymist_task::ExportHtmlTask;
pub type StaticHtmlExport = tinymist_task::HtmlExport;
pub type ExportHtmlTask = tinymist_task::ExportHtmlTask;

pub struct HtmlOutputExport;

impl<F: CompilerFeat> ExportComputation<F, TypstHtmlDocument> for HtmlOutputExport {
    type Output = HtmlOutput;
    type Config = ExportHtmlTask;

    fn run(
        _graph: &Arc<WorldComputeGraph<F>>,
        doc: &Arc<TypstHtmlDocument>,
        _config: &ExportHtmlTask,
    ) -> Result<HtmlOutput> {
        Ok(static_html(doc)?)
    }
}

pub struct HtmlOutput {
    pretty: bool,
    document: Arc<TypstHtmlDocument>,
    head_idx: Option<usize>,
    body_idx: Option<usize>,

    body: OnceLock<SourceResult<String>>,
    html: OnceLock<SourceResult<String>>,
}

impl HtmlOutput {
    fn root_child(&self, idx: Option<usize>) -> Option<&HtmlElement> {
        match self.document.root.children.get(idx?)? {
            HtmlNode::Element(e) => Some(e),
            _ => None,
        }
    }

    fn head(&self) -> Option<&HtmlElement> {
        self.root_child(self.head_idx)
    }

    /// Get the title of the document, if any.
    pub fn title(&self) -> Option<&EcoString> {
        self.head()?
            .children
            .iter()
            .find_map(|node| match node {
                HtmlNode::Element(e) if e.tag == tag::title => e.children.first(),
                _ => None,
            })
            .and_then(|node| match node {
                HtmlNode::Text(text, _) => Some(text),
                _ => None,
            })
    }

    /// Get the description of the document, if any.
    pub fn description(&self) -> Option<&EcoString> {
        self.head()?.children.iter().find_map(|node| match node {
            HtmlNode::Element(e) if e.tag == tag::meta => {
                let mut name = false;
                let mut description = None;

                for (attr, value) in &e.attrs.0 {
                    let attr = attr.resolve();
                    match attr.as_str() {
                        "name" => {
                            name |= value == "description";
                        }
                        "content" => description = Some(value),
                        _ => {}
                    }
                }

                if name {
                    description
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn body(&self) -> SourceResult<&str> {
        self.body
            .get_or_init(|| {
                let mut w = Writer {
                    pretty: self.pretty,
                    ..Writer::default()
                };
                write_indent(&mut w);
                if let Some(body) = self.root_child(self.body_idx) {
                    write_element_with_tag(&mut w, body, "div")?;
                }

                if w.pretty {
                    w.buf.push('\n');
                }

                Ok(w.buf)
            })
            .as_ref()
            .map(|s| s.as_str())
            .map_err(|e| e.clone())
    }

    pub fn html(&self) -> SourceResult<&str> {
        self.html
            .get_or_init(|| {
                let mut w = Writer {
                    pretty: self.pretty,
                    ..Writer::default()
                };
                w.buf.push_str("<!DOCTYPE html>\n");
                write_indent(&mut w);
                write_element(&mut w, &self.document.root)?;
                if w.pretty {
                    w.buf.push('\n');
                }
                Ok(w.buf)
            })
            .as_ref()
            .map(|s| s.as_str())
            .map_err(|e| e.clone())
    }

    #[cfg(feature = "hast")]
    pub fn hast(&self) -> SourceResult<reflexo_typst2hast::hast::HastElementContent> {
        reflexo_typst2hast::hast(&self.document)
    }
}

fn find_tag_child(element: &HtmlElement, tag: HtmlTag) -> Option<usize> {
    element.children.iter().position(|node| match node {
        HtmlNode::Element(e) => e.tag == tag,
        _ => false,
    })
}

/// Encodes an HTML document into a string.
pub fn static_html(document: &Arc<TypstHtmlDocument>) -> SourceResult<HtmlOutput> {
    let head_idx = find_tag_child(&document.root, tag::head);
    let body_idx = find_tag_child(&document.root, tag::body);

    Ok(HtmlOutput {
        pretty: true,
        document: document.clone(),
        head_idx,
        body_idx,
        body: OnceLock::new(),
        html: OnceLock::new(),
    })
}

#[derive(Default)]
struct Writer {
    /// The output buffer.
    buf: String,
    /// The current indentation level
    level: usize,
    /// Whether pretty printing is enabled.
    pretty: bool,
}

/// Write a newline and indent, if pretty printing is enabled.
fn write_indent(w: &mut Writer) {
    if w.pretty {
        w.buf.push('\n');
        for _ in 0..w.level {
            w.buf.push_str("  ");
        }
    }
}

/// Encode an HTML node into the writer.
fn write_node(w: &mut Writer, node: &HtmlNode) -> SourceResult<()> {
    match node {
        HtmlNode::Tag(_) => {}
        HtmlNode::Text(text, span) => write_text(w, text, *span)?,
        HtmlNode::Element(element) => write_element(w, element)?,
        HtmlNode::Frame(frame) => write_frame(w, &frame.inner),
    }
    Ok(())
}

/// Encode plain text into the writer.
fn write_text(w: &mut Writer, text: &str, span: Span) -> SourceResult<()> {
    for c in text.chars() {
        if charsets::is_valid_in_normal_element_text(c) {
            w.buf.push(c);
        } else {
            write_escape(w, c).at(span)?;
        }
    }
    Ok(())
}

/// Encode one element into the write.
fn write_element(w: &mut Writer, element: &HtmlElement) -> SourceResult<()> {
    write_element_with_tag(w, element, &element.tag.resolve())
}

/// Encode one element into the write.
fn write_element_with_tag(w: &mut Writer, element: &HtmlElement, tag: &str) -> SourceResult<()> {
    w.buf.push('<');
    w.buf.push_str(tag);

    for (attr, value) in &element.attrs.0 {
        w.buf.push(' ');
        w.buf.push_str(&attr.resolve());
        w.buf.push('=');
        w.buf.push('"');
        for c in value.chars() {
            if charsets::is_valid_in_attribute_value(c) {
                w.buf.push(c);
            } else {
                write_escape(w, c).at(element.span)?;
            }
        }
        w.buf.push('"');
    }

    w.buf.push('>');

    if tag::is_void(element.tag) {
        return Ok(());
    }

    let pretty = w.pretty;
    if !element.children.is_empty() {
        let pretty_inside = allows_pretty_inside(element.tag)
            && element.children.iter().any(|node| match node {
                HtmlNode::Element(child) => wants_pretty_around(child.tag),
                _ => false,
            });

        w.pretty &= pretty_inside;
        let mut indent = w.pretty;

        w.level += 1;
        for c in &element.children {
            let pretty_around = match c {
                HtmlNode::Tag(_) => continue,
                HtmlNode::Element(child) => w.pretty && wants_pretty_around(child.tag),
                HtmlNode::Text(..) | HtmlNode::Frame(_) => false,
            };

            if core::mem::take(&mut indent) || pretty_around {
                write_indent(w);
            }
            write_node(w, c)?;
            indent = pretty_around;
        }
        w.level -= 1;

        write_indent(w);
    }
    w.pretty = pretty;

    w.buf.push_str("</");
    w.buf.push_str(tag);
    w.buf.push('>');

    Ok(())
}

/// Whether we are allowed to add an extra newline at the start and end of the
/// element's contents.
///
/// Technically, users can change CSS `display` properties such that the
/// insertion of whitespace may actually impact the visual output. For example,
/// <https://www.w3.org/TR/css-text-3/#example-af2745cd> shows how adding CSS
/// rules to `<p>` can make it sensitive to whitespace. For this reason, we
/// should also respect the `style` tag in the future.
fn allows_pretty_inside(tag: HtmlTag) -> bool {
    (tag::is_block_by_default(tag) && tag != tag::pre)
        || tag::is_tabular_by_default(tag)
        || tag == tag::li
}

/// Whether newlines should be added before and after the element if the parent
/// allows it.
///
/// In contrast to `allows_pretty_inside`, which is purely spec-driven, this is
/// more subjective and depends on preference.
fn wants_pretty_around(tag: HtmlTag) -> bool {
    allows_pretty_inside(tag) || tag::is_metadata(tag) || tag == tag::pre
}

/// Escape a character.
fn write_escape(w: &mut Writer, c: char) -> StrResult<()> {
    // See <https://html.spec.whatwg.org/multipage/syntax.html#syntax-charref>
    match c {
        '&' => w.buf.push_str("&amp;"),
        '<' => w.buf.push_str("&lt;"),
        '>' => w.buf.push_str("&gt;"),
        '"' => w.buf.push_str("&quot;"),
        '\'' => w.buf.push_str("&apos;"),
        c if charsets::is_w3c_text_char(c) && c != '\r' => {
            write!(w.buf, "&#x{:x};", c as u32).unwrap()
        }
        _ => bail!("the character {} cannot be encoded in HTML", c.repr()),
    }
    Ok(())
}

/// Encode a laid out frame into the writer.
fn write_frame(w: &mut Writer, frame: &Frame) {
    // FIXME: This string replacement is obviously a hack.
    let svg = typst_svg::svg_frame(frame)
        .replace("<svg class", "<svg style=\"overflow: visible;\" class");
    w.buf.push_str(&svg);
}
