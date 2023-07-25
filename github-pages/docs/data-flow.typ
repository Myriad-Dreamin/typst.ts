#import "/contrib/typst/diagram.typ": node, arr, commutative_diagram

#let data-flow-graph = commutative_diagram(
  node_padding: (70pt, 50pt),
  node((0, 0), [
    Typst Documents
  ]),
  node((0, 2), [
    Preprocessed Artifact
  ]),
  node((1, 1), [
    #link("https://developer.mozilla.org/en-US/docs/Web/SVG")[Svg Document] ( `<svg/>` )
  ]),
  node((2, 1), [
    #link("https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas")[Canvas] ( `<canvas/>` )
  ]),
  arr((0, 0), (0, 2), [
    #set text(fill: green)
    `precompile with theme and screen settings`
  ]),
  arr((0, 0), (1, 1), label_pos: 0.8em, {
    set text(fill: green)
    rotate(17deg)[
      `compile to svg`
      #set text(fill: blue)
      #h(-0.5em) $space^dagger$
    ]
  }),
  arr((0, 0), (2, 1), label_pos: -0.6em, curve: -25deg, {
    set text(fill: blue)
    rotate(35deg)[`directly render` #h(-0.5em) $ space^(dagger dot.c dagger.double)$]
  }),
  arr((0, 2), (1, 1), label_pos: -0.8em, {
    set text(fill: blue)
    rotate(-17deg)[`render to svg` #h(-0.5em) $ space^dagger.double$]
  }),
  arr((1, 1), (2, 1), []),
  arr((0, 2), (2, 1), label_pos: 0.6em, curve: 25deg, {
    set text(fill: blue)
    rotate(-35deg)[`render to canvas` #h(-0.5em) $ space^(dagger.double)$]
  }),
)
