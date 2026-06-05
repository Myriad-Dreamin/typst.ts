
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test multi-recursion with nested lists.
#set rect(inset: 3pt)
#show list: rect.with(stroke: blue)
#show list: rect.with(stroke: red)
#show list: block

- List
  - Nested
  - List
- Recursive!