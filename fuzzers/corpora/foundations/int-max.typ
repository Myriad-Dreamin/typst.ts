
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(int.max, 9223372036854775807)
#test(int.max, 0x7FFFFFFFFFFFFFFF)