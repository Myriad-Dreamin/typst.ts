
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 80pt, height: 30pt)
First
#pagebreak(to: "odd")
Third
#pagebreak(to: "even")
Fourth
#pagebreak(to: "even")
Sixth
#pagebreak()
Seventh
#pagebreak(to: "odd")
#page[Nineth]
