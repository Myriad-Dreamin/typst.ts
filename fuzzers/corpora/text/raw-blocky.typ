
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test various raw parsing edge cases.

#let empty = (
  name: "empty",
  input: ``,
  text: "",
  block: false,
)

#let empty-spaces = (
  name: "empty-spaces",
  input: ```   ```,
  text: "",
  block: false,
)

#let empty-newlines = (
  name: "empty-newlines",
  input: ```


```,
  text: "\n",
  block: true,
)

#let newlines-backtick = (
  name: "newlines-backtick",
  input: ```

`

```,
  text: "\n`\n",
  block: true,
)

#let backtick = (
  name: "backtick",
  input: ``` ` ```,
  text: "`",
  block: false,
)

#let lang-backtick = (
  name: "lang-backtick",
  input: ```js ` ```,
  lang: "js",
  text: "`",
  block: false,
)

// The language tag stops on space
#let lang-space = (
  name: "lang-space",
  input: ```js test ```,
  lang: "js",
  text: "test ",
  block: false,
)

// The language tag stops on newline
#let lang-newline = (
  name: "lang-newline",
  input: ```js
test
```,
  lang: "js",
  text: "test",
  block: true,
)

// The first line and the last line are ignored
#let blocky = (
  name: "blocky",
  input: {
```
test
```
},
  text: "test",
  block: true,
)

// A blocky raw should handle dedents
#let blocky-dedent = (
  name: "blocky-dedent",
  input: {
```
 test
 ```
  },
  text: "test",
  block: true,
)

// When there is content in the first line, it should exactly eat a whitespace char.
#let blocky-dedent-firstline = (
  name: "blocky-dedent-firstline",
  input: ``` test
  ```,
  text: "test",
  block: true,
)

// When there is content in the first line, it should exactly eat a whitespace char.
#let blocky-dedent-firstline2 = (
  name: "blocky-dedent-firstline2",
  input: ``` test
```,
  text: "test",
  block: true,
)

// The first line is not affected by dedent, and the middle lines don't consider the whitespace prefix of the first line.
#let blocky-dedent-firstline3 = (
  name: "blocky-dedent-firstline3",
  input: ``` test
     test2
  ```,
  text: "test\n   test2",
  block: true,
)

// The first line is not affected by dedent, and the middle lines don't consider the whitespace prefix of the first line.
#let blocky-dedent-firstline4 = (
  name: "blocky-dedent-firstline4",
  input: ```     test
  test2
  ```,
  text: "    test\ntest2",
  block: true,
)

#let blocky-dedent-lastline = (
  name: "blocky-dedent-lastline",
  input: ```
  test
 ```,
  text: " test",
  block: true,
)

#let blocky-dedent-lastline2 = (
  name: "blocky-dedent-lastline2",
  input: ```
  test
   ```,
  text: "test",
  block: true,
)

#let blocky-tab = (
  name: "blocky-tab",
  input: {
```
	test
```
},
  text: "\ttest",
  block: true,
)

// This one is a bit problematic because there is a trailing tab below "test"
// which the editor constantly wants to remove.
#let blocky-tab-dedent = (
  name: "blocky-tab-dedent",
  input: eval("```\n\ttest\n  \n ```"),
  text: "test\n ",
  block: true,
)

#let extra-first-line-ws = (
  name: "extra-first-line-ws",
  input: eval("```   \n```"),
  text: "",
  block: true,
)

#let cases = (
  empty,
  empty-spaces,
  empty-newlines,
  newlines-backtick,
  backtick,
  lang-backtick,
  lang-space,
  lang-newline,
  blocky,
  blocky-dedent,
  blocky-dedent-firstline,
  blocky-dedent-firstline2,
  blocky-dedent-firstline3,
  blocky-dedent-lastline,
  blocky-dedent-lastline2,
  blocky-tab,
  blocky-tab-dedent,
  extra-first-line-ws,
)

#for c in cases {
  let block = c.block
  assert.eq(c.text, c.input.text, message: "in point " + c.name + ", expect " + repr(c.text) + ", got " + repr(c.input.text) + "")
  assert.eq(block, c.input.block, message: "in point " + c.name + ", expect " + repr(block) + ", got " + repr(c.input.block) + "")
}