
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// General footer-only tests
#set page(height: 9em)
#table(
  columns: 2,
  [a], [],
  [b], [],
  [c], [],
  [d], [],
  [e], [],
  table.footer(
    [*Ok*], table.cell(rowspan: 2)[test],
    [*Thanks*]
  )
)