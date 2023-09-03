
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test format auto detect
#image.decode(read("/assets/files/tiger.jpg", encoding: none), width: 80%)
