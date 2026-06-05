
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#let c = counter("c")
#let s = context c.display() + c.step()
#let tree = [درخت]
#let line = [A #s B #tree #s #tree #s #tree C #s D #s]
#line \
#line