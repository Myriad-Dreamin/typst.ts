
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test named argument overriding with the spread operator.
#let check(it, s) = test(it.body.text, repr(s))
#let func(a: 1, b: 1) = (a: a, b: b)
#let dict = (a: 2, b: 2)
#let args = arguments(a: 3, b: 3)
#check($func()$, (a: 1, b: 1))
#check($func(..dict, ..args)$, (a: 3, b: 3))
#check($func(..args, ..dict)$, (a: 2, b: 2))
#check($func(a: #4, ..dict, b: #4)$, (a: 2, b: 4))
#check($func(a: #4, ..args, b: #4)$, (a: 3, b: 4))