
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, a row of "-" only should have a very small height; but
// after adding an alignment point "&", the row gains a larger height.
// We need to test alignment point "&" does not affect a row's height.
#set stack(dir: ltr, spacing: 0.5em)
#stack(
  box($ - - $, fill: silver),
  box($ - - $, fill: silver)
)

#stack(
  box($ a \ - - $, fill: silver),
  box($ &- - \ &a $, fill: silver),
  box($ &a \ &- - $, fill: silver)
)