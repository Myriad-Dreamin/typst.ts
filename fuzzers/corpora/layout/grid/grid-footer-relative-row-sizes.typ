
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Relative lengths
#set page(height: 10em)
#table(
  rows: (30%, 30%, auto),
  [C],
  [C],
  table.footer[*A*][*B*],
)