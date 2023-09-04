
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test loading different image formats.

// Load an RGBA PNG image.
#image("/assets/files/rhino.png")

// Load an RGB JPEG image.
#set page(height: 60pt)
#image("/assets/files/tiger.jpg")
