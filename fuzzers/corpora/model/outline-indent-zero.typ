
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.a.")
#show heading: none

#outline(indent: 0pt)

= A
== B
=== C
==== Title that breaks across lines
#set heading(numbering: none)
== E
= F