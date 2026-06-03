
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Mixing raw
#set text(lang: "he")
לדוג. `if a == b:` זה תנאי
#set raw(lang: "python")
לדוג. `if a == b:` זה תנאי

#show raw: set text(dir:rtl)
לתכנת בעברית `אם א == ב:`