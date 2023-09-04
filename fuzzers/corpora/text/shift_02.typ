
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set underline(stroke: 0.5pt, offset: 0.15em)
#underline[The claim#super[\[4\]]] has been disputed. \
The claim#super[#underline[\[4\]]] has been disputed. \
It really has been#super(box(text(baseline: 0pt, underline[\[4\]]))) \
