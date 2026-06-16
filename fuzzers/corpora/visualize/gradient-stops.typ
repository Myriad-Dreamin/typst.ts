
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(gradient.linear(red, green, blue).stops(), ((red, 0%), (green, 50%), (blue, 100%)))