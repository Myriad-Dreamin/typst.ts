
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test importing from function scopes.

#import enum: item
#import assert.with(true): *

#enum(
   item(1)[First],
   item(5)[Fifth]
)
#eq(10, 10)
#ne(5, 6)