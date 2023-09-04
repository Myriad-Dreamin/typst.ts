
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test using ems in arbitrary places.
#set text(size: 5pt)
#set text(size: 2em)
#set square(fill: red)

#let size = {
  let size = 0.25em + 1pt
  for _ in range(3) {
    size *= 2
  }
  size - 3pt
}

#stack(dir: ltr, spacing: 1fr, square(size: size), square(size: 25pt))
