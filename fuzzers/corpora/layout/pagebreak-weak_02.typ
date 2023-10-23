
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// After only ignorables, but regular break
// Should result in three pages.
First
#pagebreak()
#counter(page).update(1)
#metadata("Some")
#pagebreak()
Third
