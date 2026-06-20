
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test shadowing a variable while assigning to it and calling a method on it.
#{
  let var = "a"
  var += var.at(0, default: let var = "b")
  test(var, "ba")
}