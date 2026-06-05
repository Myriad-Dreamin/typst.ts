
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When the header is the last grid child, it shouldn't include the gutter row
// after it, because there is none.
#grid(
  columns: 2,
  gutter: 3pt,
  grid.header(
    [a], [b],
    [c], [d]
  )
)