
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Automatic position cell skips custom position cell
#grid(
  grid.cell(x: 0, y: 0)[This shall not error],
  [A]
)