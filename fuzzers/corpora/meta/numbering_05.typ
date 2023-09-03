
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(lang: "ko", font: ("Linux Libertine", "Noto Serif CJK KR"))
#for i in range(0, 4) {
  numbering("가", i)
  [ (or ]
  numbering("ㄱ", i)
  [) for #i \ ]
}
... \
#for i in range(47, 51) {
  numbering("가", i)
  [ (or ]
  numbering("ㄱ", i)
  [) for #i \ ]
}
... \
#for i in range(2256, 2260) {
  numbering("ㄱ", i)
  [ for #i \ ]
}
