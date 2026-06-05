
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that HTML frames are transparent in layout. This is less important for
// actual paged export than for _nested_ HTML frames, which take the same code
// path.
#html.frame[A]