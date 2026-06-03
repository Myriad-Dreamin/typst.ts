
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 150pt)
#set heading(numbering: "1.")

#show outline.entry.where(level: 1): set block(above: 12pt)
#show outline.entry.where(level: 1): strong

#outline(indent: auto)

#show heading: none
= Introduction
= Background
== History
== State of the Art
= Analysis
== Setup