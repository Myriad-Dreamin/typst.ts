
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test fallback case where the matching numbering is determined from the style
// chain instead of the element. Because there is no heading element at `<at>`,
// we fall back to the style that is current at the counter display, not the
// style that was current at the location. Ideally, we'd also handle this, but
// its not trivial and this should be very rare.
#set heading(numbering: "A)")
= Hello
#metadata(none) <at>
#set heading(numbering: "I)")
#context counter(heading).display(at: <at>)