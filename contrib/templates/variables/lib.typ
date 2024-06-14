//! This is a stub package for typst.ts's dyn-svg controlling the layout
//! Here is a good example of dynamic layout template: <https://github.com/Myriad-Dreamin/shiroa/blob/308e0aacc2578e9a0c424d20332c6711d1df8d1c/contrib/typst/gh-pages.typ>

/// default target is "pdf", typst.ts will set it to "web" when rendering to a
/// dynamic layout
/// Example:
/// ```typc
/// #let is-web-target() = target.starts-with("web")
/// #let is-pdf-target() = target.starts-with("pdf")
/// ```
#let target = "pdf"

/// It is in default A4 paper size
/// example:
/// ```typc
/// set page(
///   width: page-width,
///   height: auto, // Also, for a website, we don't need pagination.
/// ) if is-web-target;
/// ```
#let page-width = 595.28pt
