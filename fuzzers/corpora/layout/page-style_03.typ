
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Empty with multiple page styles.
// Should result in one eastern-colored A11 page.
#set page("a4")
#set page("a5")
#set page("a11", flipped: true, fill: eastern)
#set text(font: "Roboto", white)
#smallcaps[Typst]
