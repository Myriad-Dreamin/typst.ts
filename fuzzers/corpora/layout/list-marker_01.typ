
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that items are cycled.
#set list(marker: ([--], [•]))
- A
  - B
    - C
