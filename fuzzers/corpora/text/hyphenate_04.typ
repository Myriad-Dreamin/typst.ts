
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// This sequence would confuse hypher if we passed trailing / leading
// punctuation instead of just the words. So this tests that we don't
// do that. The test passes if there's just one hyphenation between
// "net" and "works".
#set page(width: 60pt)
#set text(hyphenate: true)
#h(6pt) networks, the rest.
