
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Make sure that the `arrow.l.r` symbol correctly works as an accent even
// though it includes a text presentation variation selector.

// Ensure that symbol style works.
$ accent(x + y, arrow.l.r) $
// Ensure that string style works.
$ accent(x + y, "↔") $
// Ensure that content style works.
$ accent(x + y, ↔) $
// Ensure that shorthand style works.
$ accent(x + y, <->) $
// Ensure that function call works.
$ arrow.l.r(x + y) $