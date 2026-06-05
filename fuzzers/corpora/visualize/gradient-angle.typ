
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(gradient.linear(red, green).angle(), 0deg)
#test(gradient.linear(red, green, dir: ltr).angle(), 0deg)
#test(gradient.linear(red, green, dir: rtl).angle(), 180deg)
#test(gradient.linear(red, green, dir: ttb).angle(), 90deg)
#test(gradient.linear(red, green, dir: btt).angle(), 270deg)