
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that image measurement doesn't turn `inf / some-value` into 0pt.
#context {
  let size = measure(image("/assets/images/tiger.jpg"))
  test(size, (width: 1024pt, height: 670pt))
}