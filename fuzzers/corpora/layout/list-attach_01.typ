
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that attached list isn't affected by block spacing.
#show list: set block(above: 100pt)
Hello
- A
World
- B
