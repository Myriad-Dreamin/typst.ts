
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: auto)
- a
- #align(bottom)[b]
- c

d

#set page(height: 10em)
- a
- #align(bottom)[b]
- c

d