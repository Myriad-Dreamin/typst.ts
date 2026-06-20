
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Highlighting identifiers, field accesses and function calls in math
#set page(width: auto)
```typm
hello
hello-world
hello()
box[]
hello.world
hello.world()
hello-world()
hello_world()
hello.my.world()
emph(hello.my.world())
emph(hello.my().world)
emph(hello.my().world())
emph (hello.my().world())
#hello
#hello()
#hello.world
#hello.world()
#box[]
```