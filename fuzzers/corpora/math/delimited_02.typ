
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test manual matching.
$ [|a/b|] != lr(|]a/b|]) != [a/b) $
$ lr(| ]1,2\[ + 1/2|) $
