
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test successful assertions.
#assert(5 > 3)
#assert.eq(15, 15)
#assert.ne(10, 12)