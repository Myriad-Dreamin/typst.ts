
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.a.")
#show heading: none

#outline(indent: 1em)

= A
== B
=== C
==== Title that breaks
#set heading(numbering: none)
== E
= F