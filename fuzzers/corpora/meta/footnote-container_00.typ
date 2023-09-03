
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test footnote in caption.
Read the docs #footnote[https://typst.app/docs]!
#figure(
  image("/assets/files/graph.png", width: 70%),
  caption: [
    A graph #footnote[A _graph_ is a structure with nodes and edges.]
  ]
)
More #footnote[just for ...] footnotes #footnote[... testing. :)]
