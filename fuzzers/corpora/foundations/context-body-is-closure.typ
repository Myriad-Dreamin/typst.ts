
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Regression test since this used to be a hard crash.
#(context (a: none) => {})