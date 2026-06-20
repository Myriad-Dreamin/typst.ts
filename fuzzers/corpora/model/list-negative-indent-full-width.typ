
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 200pt)
#[
  #set list(indent: -50pt)
  - #lorem(12)
]
#[
  #set list(marker: box(width: 100%, height: 1em, fill: red))
  - abc
  #set list(indent: -50pt)
  - abc
]