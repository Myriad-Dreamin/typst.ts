
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 200pt)
#show raw: it => stack(dir: ttb, ..it.lines)
#show raw.line: it => {
  box(
    width: 100%,
    height: 1.75em,
    inset: 0.25em,
    fill: if calc.rem(it.number, 2) == 0 {
      luma(90%)
    } else {
      white
    },
    align(horizon, stack(
      dir: ltr,
      box(width: 15pt)[#it.number],
      it.body,
    ))
  )
}

```typ
#show raw.line: block.with(
  fill: luma(60%)
);

Hello, world!

= A heading for good measure
```
