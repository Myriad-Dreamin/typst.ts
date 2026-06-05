
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  columns: (1fr, 1fr),
  lines(6),
  [
    #text(lang: "ar", font: ("Libertinus Serif", "Noto Sans Arabic"))[مجرد نص مؤقت لأغراض العرض التوضيحي. ]
    #text(lang: "ar")[سلام]
  ],
)