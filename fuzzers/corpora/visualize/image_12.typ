
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test format manual
#image.decode(read("/assets/files/tiger.jpg", encoding: none), format: "jpg", width: 80%)
