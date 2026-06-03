
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that metadata doesn't prevent trailing spaces from being trimmed.
#set par(justify: true, linebreaks: "simple")
#set text(hyphenate: false)
Lorem ipsum dolor #metadata(none) nonumy eirmod tempor.