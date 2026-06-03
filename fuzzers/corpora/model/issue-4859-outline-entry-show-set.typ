
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.a.")
#show outline.entry.where(level: 1): set outline.entry(fill: none)
#show heading: none

#outline()

= A
== B