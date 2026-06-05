
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show heading: it => {
  set page(numbering: "1", margin: (bottom: 20pt))
  it
}

= Introduction