
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Weak fr survives with strong fr, like weak rel survives with strong rel.
#set page(height: 100pt)
#v(1fr, weak: false)
#v(1fr, weak: true)
2
#v(1fr, weak: false)
#v(2fr, weak: true)
4