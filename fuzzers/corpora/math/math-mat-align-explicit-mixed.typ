// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test explicit alignment in some columns with align parameter in a matrix.
#let data = (
  ($&18&&.02$, $1$, $+1$),
  ($-&9&&.3$, $-1$, $-&21$),
  ($&&&.011$, $1$, $&0$)
)
$ #math.mat(align: left, ..data) $
$ #math.mat(align: center, ..data) $
$ #math.mat(align: right, ..data) $