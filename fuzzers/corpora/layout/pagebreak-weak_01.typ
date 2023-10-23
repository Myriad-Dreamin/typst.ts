
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// After only ignorables & invisibles
// Should result in two pages.
First
#pagebreak(weak: true)
#counter(page).update(1)
#metadata("Some")
#pagebreak(weak: true)
Second
