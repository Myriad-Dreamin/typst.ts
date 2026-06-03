
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test show rule in function.
#let starwars(body) = {
  show list: it => block({
    stack(dir: ltr,
      text(red, it),
      1fr,
      scale(x: -100%, text(blue, it)),
    )
  })
  body
}

- Normal list

#starwars[
  - Star
  - Wars
  - List
]

- Normal list