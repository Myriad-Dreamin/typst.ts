
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Replace worlds but only in lists.
#show list: it => [
  #show "World": [🌎]
  #it
]

World
- World