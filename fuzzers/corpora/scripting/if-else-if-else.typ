
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test else if.

#let nth(n) = {
  str(n)
  if n == 1 { "st" }
  else if n == 2 { "nd" }
  else if n == 3 { "rd" }
  else { "th" }
}

#test(nth(1), "1st")
#test(nth(2), "2nd")
#test(nth(3), "3rd")
#test(nth(4), "4th")
#test(nth(5), "5th")