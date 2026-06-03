
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether multiple overlarge footnotes are properly split up across
// pages.
#set page(width: 20pt, height: 20pt)
#set footnote.entry(indent: 0pt)

A

#footnote(text(size: 15pt)[a] * 100)
#footnote(text(size: 15pt)[b] * 100)
#footnote[Fit]

B

C