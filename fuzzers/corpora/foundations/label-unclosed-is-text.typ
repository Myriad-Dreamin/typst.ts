
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that incomplete label is text.
1 < 2 is #if 1 < 2 [not] a label.