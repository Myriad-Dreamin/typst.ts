
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(size: 5pt)
A // 5pt
#[
  #set text(size: 2em)
  B // 10pt
  #[
    #set text(size: 1.5em + 1pt)
    C // 16pt
    #text(size: 2em)[D] // 32pt
    E // 16pt
  ]
  F // 10pt
]
G // 5pt
