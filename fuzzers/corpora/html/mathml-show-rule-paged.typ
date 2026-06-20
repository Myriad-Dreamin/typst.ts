
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 24-45 MathML element was ignored during paged export
#show math.frac: it => html.elem("mrow", it)
$ a/b $