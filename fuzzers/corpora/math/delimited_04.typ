
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that symbols aren't matched automatically.
$ bracket.l a/b bracket.r
  = lr(bracket.l a/b bracket.r) $
