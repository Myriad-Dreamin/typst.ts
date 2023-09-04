
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Verify that writing script/language combination has an effect
#{
  set text(size:20pt)
  set text(script: "latn", lang: "en")
  [Ş ]
  set text(script: "latn", lang: "ro")
  [Ş ]
  set text(script: "grek", lang: "ro")
  [Ş ]
}
