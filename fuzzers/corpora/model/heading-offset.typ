
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test setting the starting offset.
#set heading(numbering: "1.1")
#show heading.where(level: 2): set text(blue)
= Level 1

#heading(depth: 1)[We're twins]
#heading(level: 1)[We're twins]

== Real level 2

#set heading(offset: 1)
= Fake level 2
== Fake level 3