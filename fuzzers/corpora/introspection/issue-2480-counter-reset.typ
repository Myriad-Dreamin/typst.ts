
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let q = counter("question")
#let step-show =  q.step() + context q.display("1")
#let g = grid(step-show, step-show, gutter: 2pt)

#g
#pagebreak()
#step-show
#q.update(10)
#g