
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test configuring the size and fitting behaviour of images.

// Set width and height explicitly.
#box(image("/assets/images/rhino.png", width: 30pt))
#box(image("/assets/images/rhino.png", height: 30pt))

// Set width and height explicitly and force stretching.
#image("/assets/images/monkey.svg", width: 100%, height: 20pt, fit: "stretch")

// Make sure the bounding-box of the image is correct.
#align(bottom + right, image("/assets/images/tiger.jpg", width: 40pt, alt: "A tiger"))