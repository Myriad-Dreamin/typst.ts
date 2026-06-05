
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  "hi 6"
  // comment


  /* comment */
    .clusters()
}, ("h", "i", " ", "6"))