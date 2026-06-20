
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tags should not affect the row height of equations.
#set stack(dir: ltr, spacing: 0.5em)
#stack(
  box($ - - $, fill: silver),
  box($ #metadata(none) - - $, fill: silver),
)

#stack(
  box($ a \ - - $, fill: silver),
  box($ a \ #metadata(none) - - $, fill: silver),
  box($ - - \ a $, fill: silver),
  box($ #metadata(none) - - \ a $, fill: silver),
)