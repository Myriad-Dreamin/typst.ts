
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test mid size when lr size is set, when nested in an equation.
#set page(width: auto)

#let body = ${ A mid(|) integral }$
$ lr(body) quad
  lr(size: #1em, body) quad
  lr(size: #(1em+20%), body) $