
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 60pt)
#rect(inset: 0pt, columns(2)[
  Text
  #v(12pt)
  Hi
  #v(10pt, weak: true)
  At column break.
])
