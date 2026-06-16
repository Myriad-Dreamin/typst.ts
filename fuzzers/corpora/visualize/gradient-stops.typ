// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(gradient.linear(red, green, blue).stops(), ((red, 0%), (green, 50%), (blue, 100%)))
