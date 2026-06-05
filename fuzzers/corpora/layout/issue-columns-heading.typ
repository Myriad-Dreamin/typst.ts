
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The well-known columns bug.
#set page(height: 70pt)

Hallo
#columns(2)[
  = A
  Text
  = B
  Text
]