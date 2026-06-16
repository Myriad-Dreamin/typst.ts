// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test flattened accent glyph variants.
#show math.equation: set text(font: "STIX Two Math")
$hat(a) hat(A)$
$tilde(w) tilde(W)$
$grave(i) grave(j)$
$grave(I) grave(J)$