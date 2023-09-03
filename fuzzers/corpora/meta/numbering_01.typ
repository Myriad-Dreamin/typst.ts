
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#for i in range(0, 4) {
  numbering("A", i)
  [ for #i \ ]
}
... \
#for i in range(26, 30) {
  numbering("A", i)
  [ for #i \ ]
}
... \
#for i in range(702, 706) {
  numbering("A", i)
  [ for #i \ ]
}
