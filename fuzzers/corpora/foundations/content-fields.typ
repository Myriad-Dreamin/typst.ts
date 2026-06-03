
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test content fields method.
#test([a].fields(), (text: "a"))
#test([a *b*].fields(),  (children: ([a], [ ], strong[b])))