
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test replacing with custom content (asterism).
#set page(width: 200pt)
#show divider: set align(center)
#show divider: block[∗ ∗ ∗]
Chapter 1
#divider()
Chapter 2