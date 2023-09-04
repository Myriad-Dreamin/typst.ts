
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that tracking doesn't disrupt mark placement.
#set text(font: ("PT Sans", "Noto Serif Hebrew"))
#set text(tracking: 0.3em)
טֶקסט
