
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#h(0pt) #box[] <a>

#context test(query(<a>).len(), 1)