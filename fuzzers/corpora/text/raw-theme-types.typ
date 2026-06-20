
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
```typ
#let hi = "Hello World"
```

#set raw(theme: path("/assets/themes/halcyon.tmTheme"))
```typ
#let hi = "Hello World"
```

#set raw(theme: auto)
```typ
#let hi = "Hello World"
```