
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(int.min, -1 - int.max)
#test(int.min, int("-9223372036854775808"))