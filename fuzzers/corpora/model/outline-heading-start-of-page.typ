
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 140pt, height: 200pt, margin: (bottom: 20pt), numbering: "1")
#set heading(numbering: "(a/1)")
#show heading.where(level: 1): set text(12pt)
#show heading.where(level: 2): set text(10pt)

#set outline.entry(fill: none)
#outline()

= A
= B
#lines(3)

// This heading is right at the start of the page, so that we can test
// whether the tag migrates properly.
#[
  #set heading(outlined: false)
  == C
]

A

== D
== F
==== G