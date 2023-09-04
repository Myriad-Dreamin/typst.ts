
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Text inside raw block should be unaffected by outer alignment by default.
#set align(center)
#set page(width: 180pt)
#set text(6pt)

#lorem(20)

```py
def something(x):
  return x

a = 342395823859823958329
b = 324923
```

#lorem(20)
