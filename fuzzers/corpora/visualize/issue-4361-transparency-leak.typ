
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that transparency doesn't leak from shapes to images in PDF. The PNG
// test doesn't validate it, but at least we can discover regressions on the PDF
// output with a PDF comparison script.
#rect(fill: red.transparentize(50%))
#image("/assets/images/tiger.jpg", width: 45pt)