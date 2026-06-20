
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto, height: auto)
- Text
  - Text $ "O1" = (7 "O1" + 3 (display((sum_(i = 1)^4 L_i)/4)))/10 $
  - $ "O1" = (7 "O1" + 3 (display((sum_(i = 1)^4 L_i)/4)))/10 $