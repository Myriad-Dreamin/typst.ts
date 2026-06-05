
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show raw: it => eval(it.text, mode: "markup")

```
#show emph: image("/assets/images/tiger.jpg", width: 50%)
_Tiger!_
```