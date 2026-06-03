
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let a = "hello"
#let b = "world"
#let c = "value"
#let d = "conflict"

#test(((a): b), ("hello": "world"))
#test(((a): 1, (a): 2), ("hello": 2))
#test((hello: 1, (a): 2), ("hello": 2))
#test((a + b: c, (a + b): d, (a): "value2", a: "value3"), ("helloworld": "conflict", "hello": "value2", "a": "value3"))