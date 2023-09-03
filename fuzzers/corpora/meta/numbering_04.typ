
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(lang: "jp", font: ("Linux Libertine", "Noto Serif CJK JP"))
#for i in range(0, 4) {
  numbering("イ", i)
  [ (or ]
  numbering("い", i)
  [) for #i \ ]
}
... \
#for i in range(47, 51) {
  numbering("イ", i)
  [ (or ]
  numbering("い", i)
  [) for #i \ ]
}
... \
#for i in range(2256, 2260) {
  numbering("イ", i)
  [ for #i \ ]
}
