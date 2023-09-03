
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Link containing a block.
#link("https://example.com/", block[
  My cool rhino
  #box(move(dx: 10pt, image("/assets/files/rhino.png", width: 1cm)))
])
