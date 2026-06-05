
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let c = counter("mycounter")
#c.update(1)

#context [
  #c.update(2)
  #c.get() \
  Second: #context c.get()
]