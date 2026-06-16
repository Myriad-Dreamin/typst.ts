
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tags should not affect the row height of equations.
#box($ - - $, fill: silver)
#box($ #metadata(none) - - $, fill: silver) \
#box($ a \ - - $, fill: silver)
#box($ a \ #metadata(none) - - $, fill: silver)
#box($ - - \ a $, fill: silver)
#box($ #metadata(none) - - \ a $, fill: silver)