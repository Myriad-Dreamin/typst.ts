
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let func1(my: none) = my
#let func2(_my: none) = _my
#let func3(my-body: none) = my-body
#let func4(_my-body: none) = _my-body
#let func5(m: none) = m
$ func1(my: a) $
$ func2(_my: a) $
$ func3(my-body: a) $
$ func4(_my-body: a) $
$ func5(m: a) $
$ func5(m: sigma : f) $
$ func5(m: sigma:pi) $