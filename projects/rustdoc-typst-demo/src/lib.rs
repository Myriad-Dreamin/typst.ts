//! This crate demonstrates an approach to including [Typst](https://typst.app/docs/) in Rust docs. It tries to balance
//! readable source code, attractive rendered output, and ease of use.
//!
//! Docs with Typst can be generated locally and [on docs.rs](https://docs.rs/rustdoc-typst-demo).
//!
//! Setup
//! =====
//!
//! You'll only need one file: just grab `typst-header.html` from this project and put it into the
//! root of your project.
//!
//! ## Rendering Locally
//!
//! This project can be documented locally with the following commands. Dependencies are
//! documented separately because you probably don't want your dependencies' docs to use Typst.
//! Also, dependencies would not build correctly because of relative paths.
//!
//! ```bash
//! cargo doc
//! RUSTDOCFLAGS="--html-in-header typst-header.html" cargo doc --no-deps --open
//! ```
//!
//! ## Rendering on Docs.rs
//!
//! Include the following snippet in your `Cargo.toml`:
//!
//! ```toml
//! [package.metadata.docs.rs]
//! rustdoc-args = [ "--html-in-header", "typst-header.html" ]
//! ```
//!
//! How to Write Typst
//! ==================
//!
//! Here is some inline markup `{$integral f(x) dif x$}`.
//!
//! And now for a fancy math expression:
//! 
//! $ f(x) = integral_(-oo)^oo hat(f)(xi) e^(2 pi i xi x) dif x $
//! 
//! For complex markup, you can use the `typ-render`, `typc-render`, or `typm-render` directive. This is a bit more
//! robust:
//!
//! ````md
//! ```typm-render
//! f(x) = integral_(-oo)^oo hat(f)(xi) e^(2 pi i xi x) dif x
//! ```
//! ````
//!
//! ```typm-render
//! f(x) = integral_(-oo)^oo hat(f)(xi) e^(2 pi i xi x) dif x
//! ```
//!
//! ````md
//! ```typ-render
//! #import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge
//! #context {
//!   set curve(stroke: text.fill)
//!   diagram(
//!     cell-size: 15mm,
//!     $
//!       G edge(f, ->) edge("d", pi, ->>) & im(f) \
//!       G slash ker(f) edge("ur", tilde(f), "hook-->")
//!     $,
//!   )
//! }
//! ```
//! ````
//!
//! ```typ-render
//! #import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge
//! #context {
//!   set curve(stroke: text.fill)
//!   diagram(
//!     cell-size: 15mm,
//!     $
//!       G edge(f, ->) edge("d", pi, ->>) & im(f) \
//!       G slash ker(f) edge("ur", tilde(f), "hook-->")
//!     $,
//!   )
//! }
//! ```
