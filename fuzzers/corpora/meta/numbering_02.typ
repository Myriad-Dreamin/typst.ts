
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(lang: "he")
#for i in range(9, 21, step: 2) {
  numbering("א.", i)
  [ עבור #i \ ]
}
