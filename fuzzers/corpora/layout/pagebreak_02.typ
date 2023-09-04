
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Two text bodies separated with and surrounded by weak pagebreaks.
// Should result in two aqua-colored pages.
#set page(fill: aqua)
#pagebreak(weak: true)
First
#pagebreak(weak: true)
Second
#pagebreak(weak: true)
