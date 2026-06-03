
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test version fields.
#test(version(1, 2, 3).major, 1)
#test(version(1, 2, 3).minor, 2)
#test(version(1, 2, 3).patch, 3)