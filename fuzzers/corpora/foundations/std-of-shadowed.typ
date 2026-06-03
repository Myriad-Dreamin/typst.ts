
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let my-grid = grid[a][b]
#let grid = "oh no!"
#test(my-grid.func(), std.grid)