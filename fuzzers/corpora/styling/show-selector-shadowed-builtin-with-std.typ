
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let heading = "bar"
#show std.heading: it => text(fill: red, it)
= #heading