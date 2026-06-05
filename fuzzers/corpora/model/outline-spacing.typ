
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.a.")
#set outline.entry(fill: none)
#show outline.entry.where(level: 1): set block(above: 1.2em)

#outline()

#show heading: none
= A
== B
== C
= D
== E