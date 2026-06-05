
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test version constructor.

// Empty.
#test(array(version()), ())

// Plain.
#test(version(1, 2).major, 1)

// Single Array argument.
#test(version((1, 2)).minor, 2)

// Mixed arguments.
#test(version(1, (2, 3), 4, (5, 6), 7).at(5), 6)