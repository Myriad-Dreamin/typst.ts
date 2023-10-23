
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// After place
// Should result in three pages.
First
#pagebreak(weak: true)
#place(right)[placed A]
#pagebreak(weak: true)
Third
