
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(first-line-indent: 1.5em)
#set heading(numbering: "1.1.a.")
#show outline.entry.where(level: 1): strong

#outline()

#show heading: none
= Introduction
= Background
== History
== State of the Art
= Analysis
== Setup