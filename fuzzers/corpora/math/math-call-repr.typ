
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(a)$, "arguments([a])")
#check($args(a,)$, "arguments([a])")
#check($args(a,b)$, "arguments([a], [b])")
#check($args(a,b,)$, "arguments([a], [b])")
#check($args(,a,b,,,)$, "arguments([], [a], [b], [], [])")