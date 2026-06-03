
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#par[
  Hello
  // Warning: 4-11 block may not occur inside of a paragraph and was ignored
  #block[]
  World
]