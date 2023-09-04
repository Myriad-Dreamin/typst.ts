
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(lang: "zh", font: ("Linux Libertine", "Noto Serif CJK SC"))
#for i in range(9,21, step: 2){
  numbering("一", i)
  [ and ]
  numbering("壹", i)
  [ for #i \ ]
}
