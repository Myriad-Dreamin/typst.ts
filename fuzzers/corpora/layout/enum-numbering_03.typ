
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test numbering with closure and nested lists.
#set enum(numbering: n => super[#n])
+ A
  + B
+ C
