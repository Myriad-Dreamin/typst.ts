
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 7em)
#table(
  columns: 3,
  [], [], table.cell(breakable: true, rowspan: 2, block(width: 2em, height: 100%, fill: red)),
  table.cell(breakable: false, block(width: 2em, height: 100%, fill: red)),
  table.cell(breakable: false, rowspan: 2, block(width: 2em, height: 100%, fill: red)),
)

// Rowspan split tests