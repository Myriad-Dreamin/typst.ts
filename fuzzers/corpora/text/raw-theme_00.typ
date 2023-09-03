
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 180pt)
#set text(6pt)
#set raw(theme: "/assets/files/halcyon.tmTheme")
#show raw: it => {
  set text(fill: rgb("a2aabc"))
  rect(
    width: 100%,
    inset: (x: 4pt, y: 5pt),
    radius: 4pt,
    fill: rgb("1d2433"),
    place(right, text(luma(240), it.lang)) + it,
  )
}

```typ
= Chapter 1
#lorem(100)

#let hi = "Hello World"
#show heading: emph
```
