
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test length fields.
#test((1pt).em, 0.0)
#test((1pt).abs, 1pt)
#test((3em).em, 3.0)
#test((3em).abs, 0pt)
#test((2em + 2pt).em, 2.0)
#test((2em + 2pt).abs, 2pt)