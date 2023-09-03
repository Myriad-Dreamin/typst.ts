
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test turning smart quotes off.
He's told some books contain questionable "example text".

#set smartquote(enabled: false)
He's told some books contain questionable "example text".
