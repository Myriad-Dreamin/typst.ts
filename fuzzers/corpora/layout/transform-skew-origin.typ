
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test setting skewing origin.
#set page(width: 100pt, height:40pt)
#set text(spacing: 20pt)
#let square = square.with(width: 8pt)
#let skew-square(origin) = box(place(square(stroke: gray))
  + place(skew(ax: -30deg, ay: -30deg, origin: origin, square())))
#skew-square(center+horizon)
#skew-square(bottom+left)
#skew-square(top+right)
#skew-square(horizon+right)