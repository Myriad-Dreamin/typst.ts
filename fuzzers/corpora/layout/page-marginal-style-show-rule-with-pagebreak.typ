
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(numbering: "1", margin: (bottom: 20pt))
#show heading: it => {
  pagebreak(weak: true)
  it
}

= Introduction