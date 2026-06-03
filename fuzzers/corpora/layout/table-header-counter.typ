
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 60pt)
#let c = counter("c")
#table(
  table.header(c.step() + context c.display()),
  [A],
  [A],
)