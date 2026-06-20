
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Highlighting for Typst markup
#set page(width: auto)
```typ
#set heading(numbering: "1.")
= Chapter 1 <chap:1>
#lorem(100)

#let hi = "Hello World"
#show heading: emph
/ Chap: @chap:1[Chapter #hi]
- *Chap:* ch--ap
+ _*Chap:*_ ch~ap
1. _Chap:_ ch---ap
```