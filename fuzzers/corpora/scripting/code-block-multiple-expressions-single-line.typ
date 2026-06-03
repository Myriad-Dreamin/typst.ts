
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Evaluates to string.
#test({ let x = "m"; x + "y" }, "my")