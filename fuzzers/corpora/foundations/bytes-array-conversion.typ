
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(array(bytes("Hello")), (0x48, 0x65, 0x6C, 0x6C, 0x6F))