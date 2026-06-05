
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that resolving is *not* taken into account.
#set line(start: (1em, 1em + 2pt))

#{
  show line.where(start: (1em, 1em + 2pt)): "Triggered"
  line()
}
#{
  show line.where(start: (10pt, 12pt)): "Not Triggered"
  line()
}