
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let t = tiling(
  size: (30pt, 30pt),
  relative: "parent",
  square(size: 30pt, fill: gradient.conic(..color.map.rainbow))
);
#set text(fill: t)

#lorem(20)