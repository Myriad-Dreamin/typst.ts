
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let foo(..body) = repr(body.pos())
#foo(a: "1", b: "2", 1, 2, 3, 4, 5, 6)
