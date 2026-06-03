
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5.5em)
#table(
  gutter: 4pt,
  table.header([L1]),
  table.header(level: 2, [L2]),
  table.footer([a])
)