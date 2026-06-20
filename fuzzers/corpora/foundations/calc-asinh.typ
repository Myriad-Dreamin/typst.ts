
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let t(a, b) = assert(calc.abs(a - b) < 1e-6)
#t(calc.asinh(0), 0.0)
#t(calc.asinh(1), calc.ln(1 + calc.sqrt(2)))