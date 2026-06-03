
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test basic stretch.
$ P -> Q stretch(->, size: #200%) R \
  R stretch(->) S stretch(->, size: #50%)^"epimorphism" T $