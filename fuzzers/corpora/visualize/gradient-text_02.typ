
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Sanity check that the direction works on text.

#set page(width: 200pt, height: auto, margin: 10pt, background: {
  rect(height: 100%, width: 30pt, fill: gradient.linear(dir: btt, red, blue))
})
#set par(justify: true)
#set text(fill: gradient.linear(dir: btt, red, blue))
#lorem(30)
