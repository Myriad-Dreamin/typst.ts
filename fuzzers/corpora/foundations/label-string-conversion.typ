
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test getting the name of a label.
#test(str(<hey>), "hey")
#test(str(label("hey")), "hey")
#test(str([Hmm<hey>].label), "hey")