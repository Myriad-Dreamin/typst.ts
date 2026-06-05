
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set super(typographic: false)
#highlight[A#super[4]] B \
A#super[#highlight[4]] B \
A#super(highlight[4]) \
#set super(typographic: true)
#highlight[A#super[4]] B \
A#super[#highlight[4]] B \
A#super(highlight[4])