
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test full numbering.
#set enum(numbering: "1.a.", full: true)
+ First
  + Nested
