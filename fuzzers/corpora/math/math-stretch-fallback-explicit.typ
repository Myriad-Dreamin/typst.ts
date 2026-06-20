
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stretching fallback when features are applied.
#show math.equation: set text(font: "STIX Two Math")
$ sscript(sqrt(a / b) quad (c / d)) $