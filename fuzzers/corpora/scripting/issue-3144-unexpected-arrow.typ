
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let f(a: 10) = a(1) + 1
#test(f(a: _ => 5), 6)