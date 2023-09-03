
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test characters.
$ a class("normal", +) b \
  a class("binary", .) b \
  lr(class("opening", \/) a/b class("closing", \\)) \
  { x class("fence", \;) x > 0} \
  a class("large", \/) b \
  a class("punctuation", :) b \
  a class("relation", ~) b \
  a + class("unary", times) b \
  class("vary", :) a class("vary", :) b $
