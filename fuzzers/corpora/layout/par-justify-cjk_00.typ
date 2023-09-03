
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test Chinese text in narrow lines.

// In Chinese typography, line length should be multiples of the character size
// and the line ends should be aligned with each other.
// Most Chinese publications do not use hanging punctuation at line end.
#set page(width: auto)
#set par(justify: true)
#set text(lang: "zh", font: "Noto Serif CJK SC")

#rect(inset: 0pt, width: 80pt, fill: rgb("eee"))[
  中文维基百科使用汉字书写，汉字是汉族或华人的共同文字，是中国大陆、新加坡、马来西亚、台湾、香港、澳门的唯一官方文字或官方文字之一。25.9%，而美国和荷兰则分別占13.7%及8.2%。近年來，中国大陆地区的维基百科编辑者正在迅速增加；
]
