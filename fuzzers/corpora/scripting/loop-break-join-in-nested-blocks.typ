
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Should output `Hello World 🌎`.
#for _ in range(10) {
  [Hello ]
  [World #{
    [🌎]
    break
  }]
}