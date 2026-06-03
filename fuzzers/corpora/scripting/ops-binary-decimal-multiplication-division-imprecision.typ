
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test digit truncation by multiplication and division.
#test(decimal("0.7777777777777777777777777777") / 1000, decimal("0.0007777777777777777777777778"))
#test(decimal("0.7777777777777777777777777777") * decimal("0.001"), decimal("0.0007777777777777777777777778"))