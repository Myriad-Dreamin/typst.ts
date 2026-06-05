
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let sym = symbol("s", ("test.basic", "s"))
#test($sym.test.basic$, $s$)