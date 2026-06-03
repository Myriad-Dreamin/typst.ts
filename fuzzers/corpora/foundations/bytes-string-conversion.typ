
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(str(bytes(range(0x41, 0x50))), "ABCDEFGHIJKLMNO")