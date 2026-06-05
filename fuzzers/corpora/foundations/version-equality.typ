
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test equality of different-length versions
#test(version(), version(0))
#test(version(0), version(0, 0))
#test(version(1, 2), version(1, 2, 0, 0, 0, 0))