
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test centered, shorter divider.
#set page(width: 200pt)
#show divider: block(
  width: 100%,
  spacing: 1em,
  align(center, line(length: 50%)),
)
Before
#divider()
After