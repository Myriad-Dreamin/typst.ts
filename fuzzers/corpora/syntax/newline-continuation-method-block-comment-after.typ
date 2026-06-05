
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 3"/* comment */
    .clusters()
}, ("h", "i", " ", "3"))