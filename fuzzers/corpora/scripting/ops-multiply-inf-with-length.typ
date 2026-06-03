
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that multiplying infinite numbers by certain units does not crash.
#(float("inf") * 1pt)
#(float("inf") * 1em)
#(float("inf") * (1pt + 1em))