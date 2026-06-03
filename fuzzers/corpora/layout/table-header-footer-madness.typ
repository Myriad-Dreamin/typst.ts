
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)
#let c = counter("c")
#let it = context c.get().first() * v(10pt)
#table(
  table.header(c.step()),
  [A],
  [A],
  [A],
  [A],
  [A],
  [A],
  [A],
  table.footer(it),
)