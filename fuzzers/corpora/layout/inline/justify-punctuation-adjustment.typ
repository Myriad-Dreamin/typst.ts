
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test punctuation marks adjustment in justified paragraph.

// The test case includes the following scenarios:
// - Compression of punctuation marks at line start or line end
// - Adjustment of adjacent punctuation marks

#set page(width: 110pt + 10pt, margin: (x: 5pt))
#set text(lang: "zh", font: "Noto Serif CJK SC")
#set par(justify: true)

标注在字间的标点符号（乙式括号省略号以外）通常占一个汉字宽度，使其易于识别、适合配置及排版，有些排版风格完全不对标点宽度进行任何调整。但是为了让文字体裁更加紧凑易读，，，以及执行3.1.4 行首行尾禁则时，就需要对标点符号的宽度进行调整。是否调整取决于……