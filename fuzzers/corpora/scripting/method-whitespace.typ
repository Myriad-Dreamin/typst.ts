
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whitespace around dot.
#test( "Hi there" . split() , ("Hi", "there"))