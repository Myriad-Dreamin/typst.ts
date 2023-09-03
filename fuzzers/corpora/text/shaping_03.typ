
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that RTL safe-to-break doesn't panic even though newline
// doesn't exist in shaping output.
#set text(dir: rtl, font: "Noto Serif Hebrew")
\ ט
