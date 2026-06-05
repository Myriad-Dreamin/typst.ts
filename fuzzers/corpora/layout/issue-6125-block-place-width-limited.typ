
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that the width of a placed block isn't limited by its siblings.
#set page(height: 70pt)
#let b = block({
  square(size: 20pt, fill: aqua)
  place(top, box(height: 10pt, width: 1fr, fill: blue))
})
#b
#b