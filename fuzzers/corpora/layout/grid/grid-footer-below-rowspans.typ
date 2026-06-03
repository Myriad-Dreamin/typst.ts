
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Footer should go below the rowspans.
#set page(margin: 2pt)
#set text(6pt)
#table(
  columns: 2,
  inset: 1.5pt,
  table.cell(rowspan: 2)[a], table.cell(rowspan: 2)[b],
  table.footer()
)