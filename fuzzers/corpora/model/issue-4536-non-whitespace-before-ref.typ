
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reference with non-whitespace before it.
#figure[] <1>
#test([(#ref(<1>))], [(@1)])