
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a spread operator followed by nothing generates two dots.
#let args(..body) = body
#let check(it, r) = test-repr(it.body.text, r)
#check($args(..)$, "arguments(sequence([.], [.]))")
#check($args(.., ..; .. , ..)$, "arguments(\n  (sequence([.], [.]), sequence([.], [.])),\n  (sequence([.], [.]), sequence([.], [.])),\n)")