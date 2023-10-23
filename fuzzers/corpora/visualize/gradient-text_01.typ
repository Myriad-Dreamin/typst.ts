
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that gradient fills on text work for globally defined gradients.

#set page(width: 200pt, height: auto, margin: 10pt, background: {
  rect(width: 100%, height: 30pt, fill: gradient.linear(red, blue))
})
#set par(justify: true)
#set text(fill: gradient.linear(red, blue))
#lorem(30)
