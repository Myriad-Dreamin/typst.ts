
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure equal decimals with different scales produce different strings.
#let f1(x) = str(x)
#let f2(x) = f1(x)
#test(f2(decimal("3.140")), "3.140")
#test(f2(decimal("3.14000")), "3.14000")