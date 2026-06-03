
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// None is spreadable.
#let f() = none
#f(..none)
#f(..if false {})
#f(..for x in () [])