
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Check that we don't require space for clearance if there is no content.
#set page(height: 100pt)
#v(1fr)
#table(
  columns: (1fr, 1fr),
  lines(2),
  [],
  lines(8),
  place(auto, float: true, block(width: 100%, height: 100%, fill: aqua))
)