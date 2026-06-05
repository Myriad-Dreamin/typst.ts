
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 150pt, numbering: "I", margin: (bottom: 20pt))
#set heading(numbering: "1.")

#set outline.entry(fill: repeat[--])
#show outline.entry.where(level: 1): it => link(
  it.element.location(),
  it.indented(it.prefix(), {
    emph(it.body())
    [ ]
    text(luma(100), box(width: 1fr, repeat[--·--]))
    [ ]
    it.page()
  })
)

#counter(page).update(3)
#outline()

#show heading: none

= Top heading
== Not top heading
=== Lower heading
=== Lower too
== Also not top

#pagebreak()
#set page(numbering: "1")

= Another top heading
== Middle heading
=== Lower heading