
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test Variants of Mainland China, Hong Kong, and Japan.

// 17 characters a line.
#set page(width: 170pt + 10pt, margin: (x: 5pt))
#set text(lang: "zh", font: "Noto Serif CJK SC")
#set par(justify: true)

孔雀最早见于《山海经》中的《海内经》：“有孔雀。”东汉杨孚著《异物志》记载，岭南：“孔雀，其大如大雁而足高，毛皆有斑纹彩，捕而蓄之，拍手即舞。”

#set text(lang: "zh", region: "hk", font: "Noto Serif CJK TC")
孔雀最早见于《山海经》中的《海内经》：「有孔雀。」东汉杨孚著《异物志》记载，岭南：「孔雀，其大如大雁而足高，毛皆有斑纹彩，捕而蓄之，拍手即舞。」
