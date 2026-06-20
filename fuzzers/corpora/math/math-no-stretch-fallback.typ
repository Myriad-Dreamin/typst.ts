
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that stretching fallback doesn't happen if the original size suffices.
#show math.equation: set text(font: "STIX Two Math")
$ script(sqrt(x) (a)) $