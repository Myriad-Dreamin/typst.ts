
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test importing items from function scopes via nested import.
#import std: grid.cell, table.cell as tcell
#test(cell, grid.cell)
#test(tcell, table.cell)