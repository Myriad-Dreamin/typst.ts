// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test item number overriding.
1. first
+ second
5. fifth

#enum(
   enum.item(1)[First],
   [Second],
   enum.item(5)[Fifth]
)