
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that metadata after spacing does not force a new paragraph.
#{
  h(1em)
  counter(heading).update(4)
  [Hello ]
  context counter(heading).display()
}