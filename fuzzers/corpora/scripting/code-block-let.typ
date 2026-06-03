
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Let evaluates to none.
#test({ let v = 0 }, none)