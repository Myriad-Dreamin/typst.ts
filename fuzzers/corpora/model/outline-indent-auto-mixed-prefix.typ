
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show heading: none
#show outline.entry.where(level: 1): strong

#outline()

#set heading(numbering: "I.i.")
= A
== B
=== Title that breaks
= C
== D
= E
#[
  #set heading(numbering: none)
  = F
  == Numberless title that breaks
  === G
]
= H