
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.a.")
#show heading: none

#outline(indent: n => (0pt, 1em, 2.5em, 3em).at(n))

= A
== B
=== C
==== Title breaks
#set heading(numbering: none)
== E
= F