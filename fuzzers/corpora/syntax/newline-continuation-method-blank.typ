
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 1"

    .clusters()
}, ("h", "i", " ", "1"))