
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that last item is repeated.
#set list(marker: ([--], [•]))
- A
  - B
    - C
