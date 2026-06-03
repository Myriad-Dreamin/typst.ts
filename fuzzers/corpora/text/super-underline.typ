
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set underline(stroke: 0.5pt, offset: 0.15em)
#set super(typographic: false)
#underline[A#super[4]] B \
A#super[#underline[4]] B \
A #underline(super[4]) B \
#set super(typographic: true)
#underline[A#super[4]] B \
A#super[#underline[4]] B \
A #underline(super[4]) B