
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether an overlarge footnote separator does not cause an infinite
// loop and compiles.
#set page(height: 2em)
#set footnote.entry(separator: v(5em))

#footnote[]