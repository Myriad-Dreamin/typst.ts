
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Another classic example.
#show "TeX": [T#h(-0.145em)#box(move(dy: 0.233em)[E])#h(-0.135em)X]
#show regex("(Lua)?(La)?TeX"): name => box(text(font: "New Computer Modern")[#name])

TeX, LaTeX, LuaTeX and LuaLaTeX!