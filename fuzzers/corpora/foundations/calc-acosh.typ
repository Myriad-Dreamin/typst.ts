
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let t(a, b) = assert(calc.abs(a - b) < 1e-6)
#t(calc.acosh(1), 0.0)
#t(calc.acosh(2), calc.ln(2 + calc.sqrt(3)))