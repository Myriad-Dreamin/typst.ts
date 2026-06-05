
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test citation in other introspection.
#set page(width: 180pt)
#set heading(numbering: "1.")

#outline(
  title: [Figures],
  target: figure.where(kind: image),
)

#pagebreak()

= Introduction <intro>
#figure(
  rect(height: 10pt),
  caption: [A pirate @arrgh in @intro],
)

#context [Citation @distress on page #here().page()]

#show bibliography: none
#bibliography("/assets/bib/works.bib", style: "chicago-shortened-notes")