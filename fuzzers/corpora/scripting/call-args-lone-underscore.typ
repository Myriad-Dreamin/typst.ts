
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that lone underscore works.
#test((1, 2, 3).map(_ => {}).len(), 3)