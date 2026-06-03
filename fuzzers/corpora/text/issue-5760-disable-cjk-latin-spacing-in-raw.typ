
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

```typ
#let hi = "你好world"
```

#show raw: set text(cjk-latin-spacing: auto)
```typ
#let hi = "你好world"
```