
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test continue while destructuring.
// Should output "one = I \ two = II \ one = I".
#for num in (1, 2, 3, 1) {
  let (word, roman) = if num == 1 {
    ("one", "I")
  } else if num == 2 {
    ("two", "II")
  } else {
    continue
  }
  [#word = #roman \ ]
}