
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#{
  "hello"
    .clusters()
  if false {

  }
  else {
    ("1", "2")
  }
}
