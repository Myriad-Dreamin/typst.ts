
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test length `to-absolute` method.
#set text(size: 12pt)
#context {
  test((6pt).to-absolute(), 6pt)
  test((6pt + 10em).to-absolute(), 126pt)
  test((10em).to-absolute(), 120pt)
}

#set text(size: 64pt)
#context {
  test((6pt).to-absolute(), 6pt)
  test((6pt + 10em).to-absolute(), 646pt)
  test((10em).to-absolute(), 640pt)
}