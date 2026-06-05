
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 4"
  // comment
    .clusters()
}, ("h", "i", " ", "4"))