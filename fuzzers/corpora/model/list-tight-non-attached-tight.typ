// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test non-attached tight list.
#set block(spacing: 15pt)
Hello
- A
World

- B
- C

More.