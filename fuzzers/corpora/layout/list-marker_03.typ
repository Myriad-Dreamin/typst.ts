
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that bare hyphen doesn't lead to cycles and crashes.
#set list(marker: [-])
- Bare hyphen is
- a bad marker
