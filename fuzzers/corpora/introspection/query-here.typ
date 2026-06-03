
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that `here()` yields the context element's location.
#context test(query(here()).first().func(), (context none).func())