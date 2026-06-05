
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test citation order in RTL text.
#set page(width: 300pt)
#set text(font: ("Libertinus Serif", "Noto Sans Arabic"))
@netwok
aaa
این است
@tolkien54
و این یکی هست
@arrgh

#bibliography("/assets/bib/works.bib")