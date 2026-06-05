
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(().join(default: "EMPTY", ", "), "EMPTY")
#test(("hello",).join(default: "EMPTY", ", "), "hello")
#test(("hello", "world").join(default: "EMPTY", ", "), "hello, world")