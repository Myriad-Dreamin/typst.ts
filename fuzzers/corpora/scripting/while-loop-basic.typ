
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Should output `2 4 6 8 10`.
#let i = 0
#while i < 10 [
  #(i += 2)
  #i
]

// Should output `Hi`.
#let iter = true
#while iter {
  iter = false
  "Hi."
}

#while false {
  dont-care
}