
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test function.
#set list(marker: n => if n == 1 [--] else [•])
- A
- B
  - C
  - D
    - E
- F