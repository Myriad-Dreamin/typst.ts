
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test RTL spacing.
#set text(dir: rtl)
A #h(10pt) B \
A #h(1fr) B
