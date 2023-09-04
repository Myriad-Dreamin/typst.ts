
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 150pt)
#set heading(numbering: "1.")

#show outline.entry.where(
  level: 1
): it => {
  v(12pt, weak: true)
  strong(it)
}

#outline(indent: auto)

#set text(8pt)
#show heading: set block(spacing: 0.65em)

= Introduction
= Background
== History
== State of the Art
= Analysis
== Setup
