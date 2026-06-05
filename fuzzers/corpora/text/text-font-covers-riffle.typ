
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Repeatedly use two fonts alternately.
#set text(font: (
  (name: "Noto Color Emoji", covers: regex("[🔗⛓‍💥]")),
  (name: "Twitter Color Emoji", covers: regex("[^🖥️]")),
  "Noto Color Emoji",
))

🔗⛓‍💥🖥️🔑

// The above should be the same as:
#{
  text(font: "Noto Color Emoji", "🔗⛓‍💥🖥️")
  text(font: "Twitter Color Emoji", "🔑")
}

// but not:
#text(font: "Twitter Color Emoji", "🔗⛓‍💥🖥️🔑")