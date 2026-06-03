
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(str({
  bytes("Hello")
  bytes((0x20,))
  bytes("World")
}), "Hello World")