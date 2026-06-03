
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.")
#show outline.entry: it => block(it.inner())
#show heading: none

#set outline.entry(fill: repeat[ -- ])
#outline()

= A
= B