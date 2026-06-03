
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test boolean operators.

// Test not.
#test(not true, false)
#test(not false, true)

// And.
#test(false and false, false)
#test(false and true, false)
#test(true and false, false)
#test(true and true, true)

// Or.
#test(false or false, false)
#test(false or true, true)
#test(true or false, true)
#test(true or true, true)

// Short-circuiting.
#test(false and dont-care, false)
#test(true or dont-care, true)