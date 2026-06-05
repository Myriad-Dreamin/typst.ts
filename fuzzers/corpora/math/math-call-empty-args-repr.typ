
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(,x,,y,,)$, "arguments([], [x], [], [y], [])")
// with whitespace/trivia:
#check($args( ,/**/x/**/, , /**/y, ,/**/, )$, "arguments([], [x], [], [y], [], [])")