
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
