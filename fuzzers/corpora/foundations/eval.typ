
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the eval function.
#test(eval("1 + 2"), 3)
#test(eval("1 + x", scope: (x: 3)), 4)
#test(eval("let x = x + 1; x + 1", scope: (x: 1)), 3)