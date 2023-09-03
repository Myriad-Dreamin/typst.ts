
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 70pt)

Hallo
#columns(2)[
  = A
  Text
  = B
  Text
]
