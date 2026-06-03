
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test capturing with unnamed function.
#let f = 10
#let f = () => f
#test(type(f()), int)