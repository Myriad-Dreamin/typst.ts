
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Empty with multiple page styles.
// Should result in a small white page.
#set page("a4")
#set page("a5")
#set page(width: 1cm, height: 1cm)
