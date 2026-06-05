
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 5"
  /*comment*/.clusters()
}, ("h", "i", " ", "5"))