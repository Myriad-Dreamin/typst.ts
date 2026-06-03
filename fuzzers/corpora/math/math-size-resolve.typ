
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let length = context repr(measure("--").width)
$ a length a ^ length $