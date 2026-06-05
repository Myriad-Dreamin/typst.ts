
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that lists are affected by correct indents.
#let fruit = [
  - Apple
  - Orange
  #list(body-indent: 20pt)[Pear]
]

- Fruit
#[#set list(indent: 10pt)
 #fruit]
- No more fruit