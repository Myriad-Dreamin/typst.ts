
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(lang: "jp", font: ("Linux Libertine", "Noto Serif CJK JP"))
#for i in range(0, 9) {
  numbering("あ", i)
  [ and ]
  numbering("I.あ", i, i)
  [ for #i \ ]
}

#for i in range(0, 9) {
  numbering("ア", i)
  [ and ]
  numbering("I.ア", i, i)
  [ for #i \ ]
}
