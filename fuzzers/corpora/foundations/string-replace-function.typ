
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `replace` method with `Func` replacements.

#test("abc".replace(regex("[a-z]"), m => {
  str(m.start) + m.text + str(m.end)
}), "0a11b22c3")
#test("abcd, efgh".replace(regex("\\w+"), m => {
  upper(m.text)
}), "ABCD, EFGH")
#test("hello : world".replace(regex("^(.+)\\s*(:)\\s*(.+)$"), m => {
  upper(m.captures.at(0)) + m.captures.at(1) + " " + upper(m.captures.at(2))
}), "HELLO : WORLD")
#test("hello world, lorem ipsum".replace(regex("(\\w+) (\\w+)"), m => {
  m.captures.at(1) + " " + m.captures.at(0)
}), "world hello, ipsum lorem")
#test("hello world, lorem ipsum".replace(regex("(\\w+) (\\w+)"), count: 1, m => {
  m.captures.at(1) + " " + m.captures.at(0)
}), "world hello, lorem ipsum")
#test("123 456".replace(regex("[a-z]+"), "a"), "123 456")

#test("abc".replace("", m => "-"), "-a-b-c-")
#test("abc".replace("", m => "-", count: 1), "-abc")
#test("123".replace("abc", m => ""), "123")
#test("123".replace("abc", m => "", count: 2), "123")
#test("a123b123c".replace("123", m => {
  str(m.start) + "-" + str(m.end)
}), "a1-4b5-8c")
#test("halla warld".replace("a", m => {
  if m.start == 1 { "e" }
  else if m.start == 4 or m.start == 7 { "o" }
}), "hello world")
#test("aaa".replace("a", m => str(m.captures.len())), "000")