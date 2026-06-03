
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show raw: it => text(font: "PT Sans", eval("[" + it.text + "]"))

Interacting
```
#set text(blue)
Blue #move(dy: -0.15em)[🌊]
```