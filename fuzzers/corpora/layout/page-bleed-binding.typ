
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(
  bleed: (inside: 10pt, outside: 5pt, top: 5pt, bottom: 5pt),
  margin: (outside: 10pt, inside: 5pt, top: 10pt, bottom: 10pt),
)

#rect(width: 100%)
#pagebreak()
#rect(width: 100%)