
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(gradient.linear(red, green, relative: "self").relative(), "self")
#test(gradient.linear(red, green, relative: "parent").relative(), "parent")
#test(gradient.linear(red, green).relative(), auto)