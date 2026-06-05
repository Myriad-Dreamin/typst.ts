
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that the fr block can also expand its parent.
#set page(height: 100pt)
#set align(center)
#block(inset: 5pt, stroke: green)[
  #rect(height: 10pt)
  #block(height: 1fr, stroke: 1pt, inset: 5pt)[
    #set align(center + horizon)
    I am the widest
  ]
  #rect(height: 10pt)
]