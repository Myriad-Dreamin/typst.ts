
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Set all margins at once.
#[
  #set page(height: 20pt, margin: 5pt)
  #place(top + left)[TL]
  #place(bottom + right)[BR]
]
