
#show link: underline
#set page(height: auto)

= Artifact streaming

A stream representation for artifact has been proposed to transfer and apply the artifact differences between the Compiler and Renderer.

== Proof of consistency

The Compiler and Renderer always keep a consistent state regarding to the artifact. Let us give a proof. Initially, both the Compiler and Renderer has a same empty artifact. When some event triggers the Compiler to update the artifact, the Compiler will generate a new artifact and send the differences between the old and new artifact to the Renderer. The Renderer will apply the differences to its artifact and then update the UI accordingly.

== Content of artifact

The artifact is considered to break into following three parts, which are transferred by different strategies.

- Full content: A subset of artifact data which does not affect the performance a lot is transferred by full content strategy, such as the source mappping.

- Tree patching: <tree-patching-strategy> A subset of the artifact which is in tree shape is transferred by tree patching strategy, such as the `typst::doc::Frame` (`SvgItem`).

- Range partition (specific to font data): <range-partition-strategy> We do send font data in granularity of glyph. To reduce transfer overhead, the streaming server should partition the glyph range into several parts and send them separately.

== Offline difference calculation

When there is no server available, a (Pre) Compiler can still calculate the differences between the documents. Then the client can determine which difference file to use according to the current document version. For examples:

- frame difference between different themes but over a same document will be preprocess using #link(<tree-patching-strategy>)[tree pathcing].

- document related font data could be shooted once when user first open the document, but the client could also preload font data by a determinsitic #link(<range-partition-strategy>)[range partition] for most likely accessed document.

== Possible implementation

A server that holds *entire current artifact data*, and also state of incremental building artifact differences (Compilation State).

```rust
/// maintains the state of the incremental compiling at server side
#[derive(Default)] 
pub struct IncrSvgDocServer { 
  /// Whether to attach debug info to the output. 
  should_attach_debug_info: bool, 

  /// Expected exact state of the current Compiler. 
  /// Initially it is None meaning no completed compilation. 
  doc_view: Option<SvgDocument>, 

  /// Maintaining document build status 
  module_builder: IncrModuleBuilder, 

  /// Optional page source mapping references. 
  page_source_mapping: Vec<SourceMappingNode>, 
} 
```

A client that holds *entire current artifact data*, and also state of typst document in DOM (UI State).

```rust
/// maintains the state of the incremental rendering at client side 
#[derive(Default)] 
pub struct IncrSvgDocClient { 
  /// Full information of the current document from server. 
  pub doc_view: SvgDocument,

  /// Optional page source mapping references. 
  pub page_source_mappping: Vec<SourceMappingNode>,

  /// Expected exact state of the current DOM. 
  /// Initially it is None meaning no any page is rendered. 
  pub dom_doc_view: Option<Pages>,

  /// Glyphs that has already committed to the DOM. 
  pub dom_glyph_status: GlyphStatus, 
} 
```

The server and client will communicate with each other by sending the differences between the current artifact and the expected artifact. The differences are in the form of `ArtifactDiff` which is defined as following:

```rust
type ArtifactDiff = FlatModule;

/// Flatten module so that it can be serialized.
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct FlatModule {
    pub metadata: Vec<ModuleMetadata>,
    pub item_pack: ItemPack,
    pub glyphs: Vec<(AbsoluteRef, FlatGlyphItem)>,
    pub layouts: Vec<(Abs, Vec<(Fingerprint, Size)>)>,
}

/// metadata that can be attached to a module.
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub enum ModuleMetadata {
    SourceMappingData(Vec<SourceMappingNode>),
    PageSourceMapping(Vec<Vec<SourceMappingNode>>),
    GarbageCollection(Vec<Fingerprint>),
}
```

The api will be extremely simple since we have the only one type of data to send, the `ArtifactDiff`.

```rust
use tokio::sync::mpsc::{channel, Receiver, Sender};

type ArtifactStream = (Sender<ArtifactDiff>, Receiver<ArtifactDiff>);

pub async fn main() {
  let (tx, rx): ArtifactStream = channel(100);
  let srv = tokio::spawn(IncrSvgDocServer::with_channel(tx));
  let cli = tokio::spawn(IncrSvgDocClient::with_channel(rx));
  srv.await.unwrap();
  cli.await.unwrap();
}

```