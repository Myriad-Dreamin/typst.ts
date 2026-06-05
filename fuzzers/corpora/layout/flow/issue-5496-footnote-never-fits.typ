
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether a footnote which is always too large would cause an infinite
// loop.
#set page(width: 20pt, height: 20pt)
#set footnote.entry(indent: 0pt)

#footnote(text(size: 15pt)[a] * 100)