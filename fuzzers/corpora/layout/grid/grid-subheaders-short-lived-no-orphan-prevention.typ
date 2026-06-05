
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// No orphan prevention for short-lived headers.
#set page(height: 8em)
#v(5em)
#grid(
  grid.header(level: 2, [b]),
  grid.header(level: 2, [c]),
  [d]
)