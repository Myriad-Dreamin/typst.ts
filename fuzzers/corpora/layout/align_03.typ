
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Ref: false
#test(type(center), alignment)
#test(type(horizon), alignment)
#test(type(center + horizon), alignment)
