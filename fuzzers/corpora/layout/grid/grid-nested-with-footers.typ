
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested table with footer should repeat both footers
#set page(height: 10em, width: auto)
#table(
  table(
    [a\ b\ c\ d],
    table.footer[b],
  ),
  table.footer[a],
)