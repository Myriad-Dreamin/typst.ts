
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a spread operator followed by nothing generates two dots.
#let args(..body) = body
#test-repr($args(..)$.body.text, "arguments(sequence([.], [.]))")
#test-repr($args(.., ..; .. , ..)$.body.text, "arguments(\n  (sequence([.], [.]), sequence([.], [.])),\n  (sequence([.], [.]), sequence([.], [.])),\n)")