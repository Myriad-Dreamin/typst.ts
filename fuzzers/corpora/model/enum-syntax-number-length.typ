
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that indentation works from the beginning of a number, not the end.

10. a
   11. b
 12. c // same level as b
  13. d // indented past c
14. e