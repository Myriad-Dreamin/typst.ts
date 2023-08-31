
#show link: underline
#set page(height: auto)
#show heading: it => {
  set text(size: (- it.level * 4pt) + 32pt)
  it
}

#import "bytefield.typ": *

  // code block setting
#show raw: it => {
  if it.block {
    rect(
      width: 100%,
      inset: (x: 4pt, y: 5pt),
      radius: 4pt,
      fill: rgb(239, 241, 243),
      [
        #set text(fill: rgb("#000000"))
        #place(right, text(luma(110), it.lang))
        #it
      ],
    )
  } else {
    it
  }
}

= Vector Format

```rs struct FlatModule``` uses variable-length tagged union to store data. The first $0~3$ bytes store the fixed string: #text(fill: green.darken(20%), [`"tsvr"`]). The bytes range from $8+32N$ to 40+32N store the $N$-th metadata. 

#bytefield(
  bits(8, fill: yellow.lighten(80%))[Magic], bits(8, fill: blue.lighten(80%))[`Tag = BuildVersion`], bits(8, fill: green.lighten(80%))[`Arc<BuildInfo>`], bits(16, fill: green.lighten(90%))[Padding], bits(8, fill: blue.lighten(80%))[`Tag = X`], bits(24, fill: green.lighten(80%))[`Value: Type(X)`], bits(8, fill: blue.lighten(80%))[`Tag = ..`], bits(24, fill: green.lighten(80%))[`Value: Type(..)`],
)

Currently there are 7 types of metadata in total:

```rs
pub enum ModuleMetadata {
    BuildVersion(Arc<BuildInfo>),
    SourceMappingData(Vec<SourceMappingNode>),
    PageSourceMapping(Arc<LayoutRegion>),
    GarbageCollection(Vec<Fingerprint>),
    Item(ItemPack),
    Glyph(Arc<GlyphPack>),
    Layout(Arc<LayoutRegion>),
}
```

The first 6 types are stable, and this upgrade accompanies the modification of the Layout format: the ```rs enum ModuleMetadata::Layout(Arc<LayoutRegion>)```. 

Note: *rkyv* guarantees appending new variant types without modifying existing variants, the binary format is backward compatible.

Points:

- We do not intend to allow other programming languages to use this _Vector Format_, so using rkyv can greatly reduce the difficulty of protocol design.

- Since we use rkyv, `Arc<T>` is essentially `rkyv::RelativePtr` for indexing, which means that the data of type `T` is stored elsewhere in the binary data.

=== Format Validation  

- Sometimes, users may accidentally try to deserialize data in other formats as _Vector Format_, in which case the `magic` segment will come in handy to provide an appropriate error message, e.g.:

  ```
  trying to deserialize bad vector data: expect head "tsvr" got "<html>\n  <"
  ```

- To ensure alignment, bytes $4~7$ are reserved as zero bytes. And they may be repurposed in the future. 

- The 0th Tag must be equal to `BuildVersion`, and its Value type must be `Arc<BuildInfo>`, which is used for stronger format detection than `magic`, for example, to lock the version between `compiler` and ` renderer`.

=== Compatibility  

- rkyv allows the types of the same variant to be upgraded backwards compatibly, and incompatible upgrades should be avoided.

- A special type: ```rs struct Deprecated(())```, can help us deprecate old type segments and give warnings. For example:
  ```rs
  type ItemPack = ItemPackV2;
  pub enum ModuleMetadata {
      Item(Deprecated),
      ItemV2(Arc<ItemPack>),
  }
  ```

- Backward compatibility is only considered between major versions to avoid too much garbage variants.

- We assume rkyv must have produced a stable binary format. The known issue is the stability across rust versions. *Without sacrificing performance*, it may consider a more stable rkyv layout in 0.5.0.

In fact, we also assume that typst documents are open source. With this assumption, we don't have to maintain backward compatibility, insteadly trying our best to ensure version consistency between compiler and renderer!

=== Major Changes since 0.4.0: `LayoutRegion` 

The original ```rs struct LayoutPack``` is modified to ```rs struct LayoutRegion```:

```rs
pub enum LayoutRegion {
    ByScalar(LayoutRegionRepr<Scalar>),
    ByStr(LayoutRegionRepr<ImmutStr>),
}
pub struct LayoutRegionRepr<T> {
    pub kind: ImmutStr,
    pub layouts: Vec<(T, LayoutRegionNode)>,
}
pub enum LayoutRegionNode {
    // next indirection
    Indirect(usize),
    // flat page layout
    Pages(Arc<Vec<Page>>),
    // source mapping node
    SourceMapping(SourceMappingNode),
}
pub struct Page {
    /// Unique hash to content
    pub content: Fingerprint,
    /// Page size for cropping content
    pub size: Size,
}
```

The main design goal is to allow nested compilation of multi-layer layouts. 

===== Simplest Case  

For `typst-preview`, there is only the simplest case: `LayoutRegionRepr::kind` is `width`, and the variant of `LayoutRegionNode` must be `Page`. As follows:

```rs
use LayoutRegionNode::*;
layout = LayoutRegionByScalar {
  kind: "width",
  layouts: {
    (350 * pt, Pages(vec![p0, p1])),
    (700 * pt, Pages(vec![p2])),
  }
}
```

===== Featured Layout

Considering high compression rate of sharing item data of `theme`, `LayoutRegion` can pack multiple themes when the extra overhead is acceptable. An example is as follows:

```rs
use LayoutRegionNode::*;
layouts = vec![
  LayoutRegionByScalar {
    kind: "theme",
    layouts: {
      ("light", Indirect(1)),
      ("dark", Indirect(2)),
    }
  }
  LayoutRegionByScalar { /* light theme */ },
  LayoutRegionByScalar { /* dark theme */ },
]
return FlatModule {
  items, // all of items collected from the light,dark theme.
  layouts,
}
```

===== Lazy Loading with HTTP Request Header `Byte-Range`  

According to preliminary experiments, github pages supports the `Byte-Range` request header:

```
curl --silent --range 20-60 https://myriad-dreamin.github.io/typst-book/guide/get-started.ayu.multi.sir.in | xxd
00000000: 6275 696c 6420 796f 7572 2062 6f6f 6b20  build your book
00000010: 616e 6420 7374 6172 7420 6120 6c6f 6361  and start a loca
00000020: 6c20 7765 6273 6572 76                   l webserv
```

Ideally, even if all the data is in one file, it can be mapped remotely like `mmap` syscall by requesting appropriate byte ranges.

+ This technology may not shine in the short term.
+ Whatever, the current ```rs struct LayoutRegion``` is designed to facilitate the implementation of remote file mapping technology based on `Byte-Range`.

Note:
+ All variants of ```rs struct LayoutRegion``` must be ```rs struct LayoutRegionRepr```, since the type `T` is hidden in the `layouts` field, no matter how `T` changes, it will not affect the layout of ```rs LayoutRegion```.

+ The specific content of ```rs struct LayoutRegion``` is re-designated as ```rs struct LayoutRegionNode``` to allow subsequent upgrades.
