
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(type(-9223372036854775808), float)