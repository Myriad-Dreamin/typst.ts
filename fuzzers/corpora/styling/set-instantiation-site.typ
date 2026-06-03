
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that text is affected by instantiation-site bold.
#let x = [World]
Hello *#x*