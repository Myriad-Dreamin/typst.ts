
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let sublime-syntax = ```yaml
%YAML 1.2
```.text + "\n---\n" + ```yaml
name: lang
file_extensions:
  - a
scope: source
contexts:
  main:
    - match: ''
```.text

#set raw(syntaxes: "/assets/syntaxes/SExpressions.sublime-syntax")
#set raw(syntaxes: path("/assets/syntaxes/SExpressions.sublime-syntax"))
#set raw(syntaxes: (
  path("/assets/syntaxes/SExpressions.sublime-syntax"),
  bytes(sublime-syntax),
))