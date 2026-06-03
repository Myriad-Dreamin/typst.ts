
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 2"// comment
    .clusters()
}, ("h", "i", " ", "2"))