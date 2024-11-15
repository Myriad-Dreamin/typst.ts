
#import "@preview/shiroa:0.1.2": *
#import "./templates/page.typ": main-color

#show: book

#let main-color = main-color

// #let section-numbers = state("book-section", ())
#let section-numbers = ()

#let get-section(level) = {
  // section-numbers.update(it => {
  //   while it.len() < level {
  //     it.push(0)
  //   }
  //   if it.len() >= level {
  //     let last-level = it.at(level - 1)
  //     it = it.slice(0, level - 1) + (last-level, )
  //   }
  // })
  // locate(loc => {
  //   let l = section-numbers.query(loc)
  //   return l.map(str).join(".")
  // })
}

#book-meta(
  title: "reflexo-typst Documentation",
  summary: [
    #prefix-chapter("introduction.typ")[Introduction]
    = Guidance
    - #chapter("get-started.typ", section: "1")[Get started]
      - #chapter("guide/all-in-one.typ", section: "1.1")[All-in-one (Simplified) Library for Browsers]
      - #chapter("guide/all-in-one-node.typ", section: "1.2")[All-in-one Library for Node.js]
    - #chapter("direction/main.typ", section: "2")[Technical Directions]
      - #chapter("direction/responsive.typ", section: "2.1")[Static, Responsive rendering]
      - #chapter("direction/incremental.typ", section: "2.2")[Streaming, Incremental rendering]
      - #chapter("direction/serverless.typ", section: "2.3")[Serverless rendering]
    - #chapter("guide/compilers.typ", section: "3")[Compilers]
      - #chapter("guide/compiler/ts-cli.typ", section: "3.1")[Command Line Interface]
      - #chapter("guide/compiler/service.typ", section: "3.2")[Compiler in Rust]
      - #chapter("guide/compiler/node.typ", section: "3.3")[Compiler in Node.js]
      - #chapter("guide/compiler/bindings.typ", section: "3.3")[Compiler in Wasm (Web)]
    - #chapter("guide/renderers.typ", section: "4")[Renderers]
      - #chapter("guide/renderer/ts-lib.typ", section: "4.1")[JavaScript/TypeScript Library]
      - #chapter("guide/renderer/node.typ", section: "4.2")[Node.js Library]
      - #chapter("guide/renderer/react.typ", section: "4.3")[React Library]
      - #chapter("guide/renderer/solid.typ", section: "4.4")[Solid Library]
      - #chapter("guide/renderer/angular.typ", section: "4.5")[Angular Library]
      - #chapter("guide/renderer/vue3.typ", section: "4.6")[Vue3 Library]
      - #chapter("guide/renderer/hexo.typ", section: "4.7")[Hexo Plugin]
    - #chapter("guide/trouble-shooting.typ", section: "5")[Trouble Shooting]
    // = Advanced development
    // - #chapter("guide/env-setup.typ", section: "5")[Environment Setup]
    //   - #chapter("guide/env-setup/renderer.typ", section: "5.1")[For Renderer]
    //   - #chapter("guide/env-setup/compiler.typ", section: "5.2")[For Compiler]
    // - #chapter("dev/shims/core.typ", section: "6")[Wasm Shim - typst.ts]
    //   - #chapter(none, section: "6.1")[Lazy Loading Wasm Modules]
    //   - #chapter("dev/shims/renderer.typ", section: "6.2")[Low-level Renderer APIs]
    //   - #chapter("dev/shims/compiler.typ", section: "6.3")[Low-level Compiler APIs]
    //   - #chapter(none, section: "6.4")[Topic: Font Loading]
    //   - #chapter(none, section: "6.5")[Topic: Tree Shaking]
    // - #chapter("dev/services/main.typ", section: "7")[Compiler service]
    //   - #chapter(none, section: "7.1")[Build a Precompiler]
    //   - #chapter("dev/services/editor-server.typ", section: "7.2")[Build a Editor Server]
    //   - #chapter("dev/services/cloud-compiler.typ", section: "7.3")[Build a Serverless Compiler]
    //   - #chapter(none, section: "7.4")[Topic: Preprocessing Dynamic Layout]
    //   - #chapter(none, section: "7.5")[Topic: Incremental Compilation]
    // - #chapter(none, section: "8")[Write your Owned Exporter]
    //   - #chapter(none, section: "8.1")[Native Exporters]
    //   - #chapter(none, section: "8.2")[Vector Representation]
    //   - #chapter(none, section: "8.3")[Topic: SVG Exporter/Renderer]
    = Deeper dive into typst.ts
    - #chapter(none, section: "5")[Core Concepts]
      - #chapter(none, section: "5.1")[Exporter Trait]
      - #chapter(none, section: "5.2")[Compiler Trait]
    - #chapter(none, section: "6")[Vector Represented Document]
      - #chapter(none, section: "6.1")[Data Structure]
      - #chapter(none, section: "6.2")[Binary Protocol]
      - #chapter(none, section: "6.3")[Render Virtual Machine]
      - #chapter(none, section: "6.4")[Topic: Partially Accessible Document]
      - #chapter(none, section: "6.5")[Topic: BBox Calculation]
      - #chapter(none, section: "6.6")[Topic: Semantic Layer Rendering]
    - #chapter(none, section: "7")[Compiler Infrastructure]
      - #chapter(none, section: "7.1")[Split World Models]
      - #chapter(none, section: "7.2")[Virtual File System]
      - #chapter(none, section: "7.3")[Font Management]
      - #chapter(none, section: "7.4")[Typst Package Registry]
      - #chapter(none, section: "7.5")[Compile Driver]
      - #chapter(none, section: "7.6")[Editor Workspace]
    = Project samples
    - #chapter(none, section: "8")[shiroa]
    - #chapter(none, section: "9")[typst-preview]
    - #chapter(none, section: "10")[hexo-renderer-typst]
    = Trouble Shooting
    - #chapter(none, section: "11")[Targeting Browser]
      - #chapter(none, section: "11.1")[Build Wasm Modules]
      - #chapter(none, section: "11.2")[typst.ts]
      - #chapter(none, section: "11.3")[tpyst.node]
      - #chapter(none, section: "11.4")[typst-ts-renderer]
      - #chapter(none, section: "11.5")[typst-ts-web-compiler]
    - #chapter(none, section: "12")[Installation]
      - #chapter(none, section: "12.1")[npm install/link]
    = References
    - #chapter(none, section: "13")[Routing to Renferences]
  ],
)

#get-book-meta()

// re-export page template
#import "/docs/cookery/templates/page.typ": project
#let book-page = project
