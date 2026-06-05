
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reading plain text files
#let data = read("/assets/text/hello.txt")
#test(data, "Hello, world!\n")