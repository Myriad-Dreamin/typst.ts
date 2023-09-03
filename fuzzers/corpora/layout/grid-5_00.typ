
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that trailing linebreak doesn't overflow the region.
#set page(height: 2cm)
#grid[
  Hello \
  Hello \
  Hello \

  World
]
