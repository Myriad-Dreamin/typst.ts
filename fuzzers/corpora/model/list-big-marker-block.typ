
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 200pt)
#block[
  #set list(indent: 100pt)
  - #lorem(12)
]
#block[
  #set list(marker: [AAAAAAAAAAAAAAAAAAAAA])
  - #lorem(12)
]