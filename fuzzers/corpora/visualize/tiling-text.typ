
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test a tiling on some text. You shouldn't be able to see the text, if you can
// then that means that the transform matrices are not being applied to the text
// correctly.
#let t = tiling(
  size: (30pt, 30pt),
  relative: "parent",
  square(size: 30pt, fill: gradient.conic(..color.map.rainbow))
);

#set page(
  width: 140pt,
  height: 140pt,
  fill: t
)

#rotate(45deg, scale(x: 50%, y: 70%, rect(
  width: 100%,
  height: 100%,
  stroke: 1pt,
)[
  #lorem(10)

  #set text(fill: t)
  #lorem(10)
]))