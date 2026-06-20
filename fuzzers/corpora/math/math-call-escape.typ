
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Escaped characters in calls.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(a\;b)$, "arguments(sequence([a], [;], [b]))")
#check($args(a\,b;c)$, "arguments((sequence([a], [,], [b]),), ([c],))")
#check($args(b\;c\,d;e)$, "arguments((sequence([b], [;], [c], [,], [d]),), ([e],))")
#check($args(a\: b)$, "arguments(sequence([a], [:], [ ], [b]))")
#check($args(a : b)$, "arguments(sequence([a], [ ], [:], [ ], [b]))")
#check($args(\..a)$, "arguments(sequence([.], [.], [a]))")
#check($args(.. a)$, "arguments(sequence([.], [.], [ ], [a]))")
#check($args(a..b)$, "arguments(sequence([a], [.], [.], [b]))")