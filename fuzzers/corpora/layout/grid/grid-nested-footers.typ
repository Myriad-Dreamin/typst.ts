
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 12em, width: auto)
#table(
  [a\ b\ c\ d],
  table.footer(table(
    [c],
    [d],
    table.footer[b],
  ))
)