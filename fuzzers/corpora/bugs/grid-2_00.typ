
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 100pt)
#grid(
  columns: (2cm, auto),
  rows: (auto, auto),
  rect(width: 100%, fill: red),
  rect(width: 100%, fill: blue),
  rect(width: 100%, height: 80%, fill: green),
  [hello \ darkness #parbreak() my \ old \ friend \ I],
  rect(width: 100%, height: 20%, fill: blue),
  polygon(fill: red, (0%, 0%), (100%, 0%), (100%, 20%))
)
