
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#par[
  Hello
  // Warning: 4-14 parbreak may not occur inside of a paragraph and was ignored
  #parbreak()
  World
]