
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test for narrow non-breaking space.
#show "_": sym.space.nobreak.narrow
0.1_g, 1_g, 10_g, 100_g, 1_000_g, 10_000_g, 100_000_g, 1_000_000_g