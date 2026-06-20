
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that stretching works in fonts with ssty variants.
#show math.equation: set text(font: "STIX Two Math")
$ lr(a / b|)_sqrt(c / d) $
$ integral_(-oo)^oo e^(-(m omega)/(2 planck) (x^2 + (2 i p)/(m omega) x)) dif x $