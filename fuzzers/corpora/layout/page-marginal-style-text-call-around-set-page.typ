
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#text(red, {
  set page(numbering: "1", margin: (bottom: 20pt))
  text(style: "italic")[Hello]
})