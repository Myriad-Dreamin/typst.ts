
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let ubuntu = (name: "Ubuntu", covers: regex("[\u{20}-\u{FFFF}]"))
#set text(font: ubuntu)
#set text(font: (ubuntu, "Ubuntu"))