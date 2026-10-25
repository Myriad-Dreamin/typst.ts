//! Tests that the incremental document server packs source mapping metadata
//! into artifacts when debug info is enabled, so that a client without the
//! typst document can resolve element paths back to source spans.

use std::borrow::Cow;

use reflexo_typst::vector::incr::{IncrDocClient, IncrDocServer};
use reflexo_typst::vector::ir::{ModuleMetadata, SourceMappingNode};
use reflexo_typst::vector::stream::BytesModuleStream;
use reflexo_typst::{
    config::entry::EntryOpts, config::CompileOpts, world::EntryReader, Bytes, TypstDocument,
    TypstSystemUniverse,
};

const MAIN_CONTENT: &str =
    "= Hello, world\n\nSome *emphasized* text for the source mapping test.\n";

/// Compiles a small entry document with embedded fonts only, so that the
/// test does not depend on any system or repository font assets.
fn compile_entry() -> (TypstSystemUniverse, TypstDocument) {
    let universe = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(std::env::temp_dir()),
        no_system_fonts: true,
        with_embedded_fonts: typst_assets::fonts().map(Cow::Borrowed).collect(),
        ..CompileOpts::default()
    })
    .unwrap();

    let doc = universe
        .snapshot_with_entry_content(Bytes::from_string(MAIN_CONTENT.to_owned()), None)
        .compile()
        .output
        .unwrap();

    (universe, TypstDocument::Paged(doc))
}

/// The file id of the memory entry resolved by `snapshot_with_entry_content`.
fn entry_file_id(universe: &TypstSystemUniverse) -> Option<typst::syntax::FileId> {
    let graph = universe.snapshot_with_entry_content(Bytes::from_string(String::new()), None);
    graph.snap.world.main_id()
}

fn has_source_mapping(metadata: &[ModuleMetadata]) -> bool {
    metadata.iter().any(|m| {
        matches!(m, ModuleMetadata::SourceMappingData(d) if !d.is_empty())
            || matches!(m, ModuleMetadata::PageSourceMapping(_))
    })
}

/// The element path kinds expected by `IncrDocClientKern::source_span`.
const TYPE_TEXT: u32 = 0;
const TYPE_GROUP: u32 = 1;
const TYPE_IMAGE: u32 = 2;
const TYPE_SHAPE: u32 = 3;
const TYPE_PAGE: u32 = 4;

/// The raw value of a detached span, which cannot be resolved to a source
/// location.
const DETACHED_SPAN: u64 = 1;

/// Descends into the group node at `group` and returns the first element
/// path (relative to the group) addressing a leaf node with an attached
/// span, along with that span.
fn descend_first_leaf(nodes: &[SourceMappingNode], group: usize) -> Option<(Vec<u32>, u64)> {
    let SourceMappingNode::Group(children) = &nodes[group] else {
        return None;
    };

    for (idx, child) in children.iter().enumerate() {
        let child = *child as usize;
        match &nodes[child] {
            SourceMappingNode::Group(_) => {
                if let Some((mut path, span)) = descend_first_leaf(nodes, child) {
                    let mut head = vec![TYPE_GROUP, idx as u32];
                    head.append(&mut path);
                    return Some((head, span));
                }
            }
            SourceMappingNode::Text(span) if *span != DETACHED_SPAN => {
                return Some((vec![TYPE_TEXT, idx as u32], *span));
            }
            SourceMappingNode::Image(span) if *span != DETACHED_SPAN => {
                return Some((vec![TYPE_IMAGE, idx as u32], *span));
            }
            SourceMappingNode::Shape(span) if *span != DETACHED_SPAN => {
                return Some((vec![TYPE_SHAPE, idx as u32], *span));
            }
            _ => {}
        }
    }

    None
}

/// Walks the client's source mapping data like a renderer would, returning
/// the first element path addressing a leaf node with an attached span,
/// along with that span.
fn first_leaf_path(client: &IncrDocClient) -> Option<(Vec<u32>, u64)> {
    let page_sources = client.page_source_mapping[0]
        .source_mapping(&client.doc.module)?
        .source_mapping();

    for (page_no, page) in page_sources.iter().enumerate() {
        let SourceMappingNode::Page(root) = page else {
            continue;
        };

        if let Some((mut path, span)) =
            descend_first_leaf(&client.source_mapping_data, *root as usize)
        {
            let mut head = vec![TYPE_PAGE, page_no as u32];
            head.append(&mut path);
            return Some((head, span));
        }
    }

    None
}

#[test]
fn test_pack_source_mapping_metadata() {
    let (universe, doc) = compile_entry();

    let mut server = IncrDocServer::default();
    server.set_should_attach_debug_info(true);
    let delta = server.pack_delta(&doc);
    let flat = BytesModuleStream::from_slice(&delta).checkout_owned();
    assert!(has_source_mapping(&flat.metadata));

    let mut client = IncrDocClient::default();
    client.merge_delta(flat);
    client.set_layout(client.doc.layouts[0].unwrap_single());

    let Some((path, expected_span)) = first_leaf_path(&client) else {
        panic!("the packed source mapping must contain an addressable leaf node");
    };

    // The client resolves the element path back to the leaf's source span,
    // exactly like the renderer's `source_span` wasm export does.
    let resolved = client.kern().source_span(&path).unwrap();
    assert_eq!(resolved, Some(format!("{expected_span:x}")));

    // The resolved span is a real typst span pointing into the entry file.
    let span = typst::syntax::Span::from_raw(std::num::NonZeroU64::new(expected_span).unwrap());
    assert!(!span.is_detached());
    assert_eq!(span.id(), entry_file_id(&universe));
}

#[test]
fn test_pack_source_mapping_disabled_by_default() {
    let (_, doc) = compile_entry();

    // The flag defaults to off, so no source mapping metadata is attached.
    let mut server = IncrDocServer::default();
    let delta = server.pack_delta(&doc);
    let flat = BytesModuleStream::from_slice(&delta).checkout_owned();
    assert!(!has_source_mapping(&flat.metadata));

    // A client without the metadata cannot resolve any element path.
    let mut client = IncrDocClient::default();
    client.merge_delta(flat);
    client.set_layout(client.doc.layouts[0].unwrap_single());
    assert_eq!(client.kern().source_span(&[TYPE_PAGE, 0]).unwrap(), None);
}
