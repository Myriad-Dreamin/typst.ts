
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test mid when lr size is set.
#set page(width: auto)

$ lr({ A mid(|) integral }) quad
  lr(size: #1em, { A mid(|) integral }) quad
  lr(size: #(1em+20%), { A mid(|) integral }) \

  lr(] A mid(|) integral ]) quad
  lr(size: #1em, ] A mid(|) integral ]) quad
  lr(size: #(1em+20%), ] A mid(|) integral ]) \

  lr(( A mid(|) integral ]) quad
  lr(size: #1em, ( A mid(|) integral ]) quad
  lr(size: #(1em+20%), ( A mid(|) integral ]) $