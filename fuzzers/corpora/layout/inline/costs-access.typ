
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set text(costs: (hyphenation: 1%, runt: 2%))
#set text(costs: (widow: 3%))
#context test(text.costs, (hyphenation: 1%, runt: 2%, widow: 3%, orphan: 100%))