
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// We allow whitespace around the dot.
#test( "Hi there" . split() , ("Hi", "there"))