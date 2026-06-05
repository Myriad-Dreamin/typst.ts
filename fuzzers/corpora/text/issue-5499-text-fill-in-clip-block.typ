
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let t = tiling(
  size: (30pt, 30pt),
  relative: "parent",
  square(
    size: 30pt,
    fill: gradient
      .conic(..color.map.rainbow),
  )
)

#block(clip: false, height: 2em, {
  text(fill: blue, "Hello")
  [ ]
  text(fill: blue.darken(20%).transparentize(50%), "Hello")
  [ ]
  text(fill: gradient.linear(..color.map.rainbow), "Hello")
  [ ]
  text(fill: t, "Hello")
})
#block(clip: true, height: 2em, {
  text(fill: blue, "Hello")
  [ ]
  text(fill: blue.darken(20%).transparentize(50%), "Hello")
  [ ]
  text(fill: gradient.linear(..color.map.rainbow), "Hello")
  [ ]
  text(fill: t, "Hello")
})