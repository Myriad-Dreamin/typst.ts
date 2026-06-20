
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let s = state("hey", "a")
#let double(it) = 2 * it

#s.update(double)
#s.update(double)
$ 2 + 3 $
#s.update(double)

Is: #context s.get(),
Was: #context {
  let it = query(math.equation).first()
  s.at(it.location())
}.