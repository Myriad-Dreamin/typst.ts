
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test again that folding is taken into account.
#set rect(width: 40pt, height: 10pt)
#set rect(stroke: blue)
#set rect(stroke: 2pt)

#{
  show rect.where(stroke: blue): "Not Triggered"
  rect()
}
#{
  show rect.where(stroke: 2pt): "Not Triggered"
  rect()
}
#{
  show rect.where(stroke: 2pt + blue): "Triggered"
  rect()
}