
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test assignment operators.

#let x = 0
#(x = 10)       #test(x, 10)
#(x -= 5)       #test(x, 5)
#(x += 1)       #test(x, 6)
#(x *= x)       #test(x, 36)
#(x /= 2.0)     #test(x, 18.0)
#(x = "some")   #test(x, "some")
#(x += "thing") #test(x, "something")