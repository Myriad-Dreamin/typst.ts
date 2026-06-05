
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "I.i.")
#set page(width: 150pt)
#show heading: none

#context test(outline.indent, auto)
#outline()

= A
== B
== C
== D
=== Title that breaks across lines
= E
== F
=== Aligned