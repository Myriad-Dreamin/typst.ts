
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show math.root: it => {
  show "√": set text(purple) if it.index == none
  it
}
$ sqrt(1/2) root(3, 1/2) $