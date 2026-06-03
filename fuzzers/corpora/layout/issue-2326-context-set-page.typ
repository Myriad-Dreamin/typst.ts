
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#context [
  #set page(fill: aqua)
  On page #here().page()
]