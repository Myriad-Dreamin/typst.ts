
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Variant 1: Prevent recursion by checking the children.
#let p = counter("p")
#let step = p.step()
#let nr = context p.display()
#show par: it => {
  if it.body.at("children", default: ()).at(0, default: none) == step {
    return it
  }
  par(step + [§#nr ] + it.body)
}

= A

B

C #parbreak() D

#block[E]

#block[F #parbreak() G]