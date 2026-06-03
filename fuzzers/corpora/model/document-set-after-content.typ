
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Document set rules can appear anywhere in top-level realization, also after
// content.
Hello
#set document(title: [Hello])