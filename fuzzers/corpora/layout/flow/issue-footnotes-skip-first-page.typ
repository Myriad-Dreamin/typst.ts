
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this issue, we would get an empty page at the beginning because footnote
// layout didn't properly check for in_last.
#set page(height: 50pt)
#footnote[A]
#footnote[B]