
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 150pt, numbering: "I", margin: (bottom: 20pt))
#set heading(numbering: "1.")
#show outline.entry.where(level: 1): it => [
  #let loc = it.element.location()
  #let num = numbering(loc.page-numbering(), ..counter(page).at(loc))
  #emph(link(loc, it.element))
  #text(luma(100), box(width: 1fr, repeat[#it.fill.body;·]))
  #link(loc, num)
]

#counter(page).update(3)
#outline(indent: auto)

#set text(8pt)
#show heading: set block(spacing: 0.65em)

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
