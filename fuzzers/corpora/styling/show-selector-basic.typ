
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Override lists.
#show list: it => "(" + it.children.map(v => v.body).join(", ") + ")"

- A
  - B
  - C
- D
- E