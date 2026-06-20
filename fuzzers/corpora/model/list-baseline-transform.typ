
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set rotate(reflow: true)
#set scale(reflow: true)
#set skew(reflow: true)

- Abc
- #rotate(90deg)[Abc]
- #rotate(180deg)[Abc]
- #scale(30%)[Abc]
- #skew(ax: 30deg)[Abc]
- #skew(ay: 30deg)[Abc]