
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(header: none, footer: none, numbering: "1")
Look, ma, no page numbers!

#pagebreak()

#set page(header: auto, footer: auto)
Default page numbers now.