
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let sym = symbol("s", ("basic", "s"))
#test($sym.basic$, $s$)