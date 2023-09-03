
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test numbering pattern.
#set enum(numbering: "(1.a.*)")
+ First
+ Second
  2. Nested
     + Deep
+ Normal
