
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(a;b)$, "arguments(([a],), ([b],))")
#check($args(a,b;c)$, "arguments(([a], [b]), ([c],))")
#check($args(a,b;c,d;e,f)$, "arguments(([a], [b]), ([c], [d]), ([e], [f]))")