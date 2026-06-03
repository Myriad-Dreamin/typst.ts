
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let p = plugin("/assets/plugins/hello.wasm")
#test(p.hello(), bytes("Hello from wasm!!!"))
#test(p.double_it(bytes("hey!")), bytes("hey!.hey!"))
#test(
  p.shuffle(bytes("value1"), bytes("value2"), bytes("value3")),
  bytes("value3-value1-value2"),
)