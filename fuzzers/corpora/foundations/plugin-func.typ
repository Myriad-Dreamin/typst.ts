
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let p = plugin("/assets/plugins/hello.wasm")
#test(type(p.hello), function)
#test(("a", "b").map(bytes).map(p.double_it), ("a.a", "b.b").map(bytes))