
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 180pt)
#set figure(placement: auto)

#figure(rect(height: 60pt), caption: [I])
#figure(rect(height: 40pt), caption: [II])
#figure(rect(), caption: [III])
A
#figure(rect(), caption: [IV])