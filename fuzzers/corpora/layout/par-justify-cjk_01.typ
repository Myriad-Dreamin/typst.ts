
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Japanese typography is more complex, make sure it is at least a bit sensible.
#set page(width: auto)
#set par(justify: true)
#set text(lang: "jp", font: ("Linux Libertine", "Noto Serif CJK JP"))
#rect(inset: 0pt, width: 80pt, fill: rgb("eee"))[
  ウィキペディア（英: Wikipedia）は、世界中のボランティアの共同作業によって執筆及び作成されるフリーの多言語インターネット百科事典である。主に寄付に依って活動している非営利団体「ウィキメディア財団」が所有・運営している。

  専門家によるオンライン百科事典プロジェクトNupedia（ヌーペディア）を前身として、2001年1月、ラリー・サンガーとジミー・ウェールズ（英: Jimmy Donal "Jimbo" Wales）により英語でプロジェクトが開始された。
]
