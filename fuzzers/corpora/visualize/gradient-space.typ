
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(gradient.linear(red, green, space: rgb).space(), rgb)
#test(gradient.linear(red, green, space: oklab).space(), oklab)
#test(gradient.linear(red, green, space: oklch).space(), oklch)
#test(gradient.linear(red, green, space: cmyk).space(), cmyk)
#test(gradient.linear(red, green, space: luma).space(), luma)
#test(gradient.linear(red, green, space: color.linear-rgb).space(), color.linear-rgb)
#test(gradient.linear(red, green, space: color.hsl).space(), color.hsl)
#test(gradient.linear(red, green, space: color.hsv).space(), color.hsv)