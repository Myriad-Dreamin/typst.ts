
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#block(outset: (:), [Hi]) // Ok
#box(radius: (:), [Hi]) // Ok