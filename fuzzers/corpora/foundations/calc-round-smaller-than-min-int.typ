
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(calc.round(decimal("-9223372036854775809.5")), decimal("-9223372036854775810"))
#test(calc.round(-9223372036854775809.5), -9223372036854775810.0)