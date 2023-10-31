
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 15em)

// In the following example, the space between 》！ and ？ should be squeezed.
// because zh-CN follows GB style
#set text(lang: "zh", region: "CN", font: "Noto Serif CJK SC")
原来，你也玩《原神》！？

// However, in the following example, the space between 》！ and ？ should not be squeezed.
// because zh-TW does not follow GB style
#set text(lang: "zh", region: "TW", font: "Noto Serif CJK TC")
原來，你也玩《原神》！ ？