// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, a row of "-" only should have a very small height; but
// after adding an alignment point "&", the row gains a larger height.
// We need to test alignment point "&" does not affect a row's height.
#box($ - - $, fill: silver)
#box($ - - $, fill: silver) \
#box($ a \ - - $, fill: silver)
#box($ &- - \ &a $, fill: silver)
#box($ &a \ &- - $, fill: silver)