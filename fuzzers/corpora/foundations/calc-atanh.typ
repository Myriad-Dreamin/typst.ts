
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let t(a, b) = assert(calc.abs(a - b) < 1e-6)
#t(calc.atanh(0), 0.0)
#t(calc.atanh(0.5), 0.5 * calc.ln(3))