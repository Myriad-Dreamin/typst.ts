
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test length unit conversions.
#let t(a, b) = assert(calc.abs(a - b) < 1e-6)

#t((500.934pt).pt(), 500.934)
#t((3.3453cm).cm(), 3.3453)
#t((4.3452mm).mm(), 4.3452)
#t((5.345in).inches(), 5.345)
#t((500.333666999pt).pt(), 500.333666999)
#t((3.523435cm).cm(), 3.523435)
#t((4.12345678mm).mm(), 4.12345678)
#t((5.333666999in).inches(), 5.333666999)
#t((4.123456789123456mm).mm(), 4.123456789123456)
#t((254cm).mm(), 2540.0)
#t((254cm).inches(), 100.0)
#t((2540mm).cm(), 254.0)
#t((2540mm).inches(), 100.0)
#t((100in).pt(), 7200.0)
#t((100in).cm(), 254.0)
#t((100in).mm(), 2540.0)
#t(5em.abs.cm(), 0.0)
#t((5em + 6in).abs.inches(), 6.0)