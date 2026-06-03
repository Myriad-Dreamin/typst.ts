
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test per-column stroke array
#let t = table(
  columns: 3,
  stroke: (red, blue, green),
  [a], [b], [c],
  [d], [e], [f],
  [h], [i], [j],
)
#t
#set text(dir: rtl)
#t