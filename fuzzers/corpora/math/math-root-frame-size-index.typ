
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test size of final frame when there is an index.
$ a root(, 3)         & a root(., 3) \
  a sqrt(3)           & a root(2, 3) \
  a root(#h(-1em), 3) & a root(123, 3) $