
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#for i in range(0, 9) {
  numbering("*", i)
  [ and ]
  numbering("I.a", i, i)
  [ for #i \ ]
}
