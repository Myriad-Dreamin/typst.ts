
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `accent` function.
$accent(ö, .), accent(v, <-), accent(ZZ, \u{0303})$