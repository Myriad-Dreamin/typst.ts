
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested tight lists should be uniformly spaced when list spacing is set.
#set list(spacing: 1.2em)
- A
  - B
  - C
- C