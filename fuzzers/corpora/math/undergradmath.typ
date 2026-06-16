// Meta data
#set document(title: "Typst Math for Undergrads", author: "johanvx")

// Margin
#set page(margin: 0.5in)

// Font size
#let scriptsize = 7pt
#let normalsize = 10pt
#let large = 12pt
#set text(size: normalsize, lang: "en")

// Some horizontal spacing
#let kern(length) = h(length, weak: true)
#let enspace = kern(0.5em)
#let qquad = h(2em)

// For table/grid, something like "lhs \enspace rhs"
#let cell(lhs, rhs) = box(lhs + enspace + rhs)
// Grid for code blocks
#set grid(columns: (2em, auto))
// Table for math-code listing
#set table(stroke: none, align: horizon + left, inset: 0pt, row-gutter: 0.45em)

// LaTeX and TeX logos
#let TeX = context (
  {
    let e = measure(text(normalsize, "E"))
    let T = "T"
    let E = text(normalsize, baseline: e.height / 2, "E")
    let X = "X"
    box(T + kern(-0.1667em) + E + kern(-0.125em) + X)
  }
)
#let LaTeX = context (
  {
    let l = measure(text(10pt, "L"))
    let a = measure(text(7pt, "A"))
    let L = "L"
    let A = text(7pt, baseline: a.height - l.height, "A")
    box(L + kern(-0.36em) + A + kern(-0.15em) + TeX)
  }
)

// Update date
#let date = "2023-07-22"

// Unavailable (last check date)
#show "??": box(text(red, [#date #emoji.crossmark]))
// Tricky
#show "!!": box(text(blue, emoji.drops))
// No idea
#show "?!": box(text(orange, [No idea #emoji.face.unhappy]))
// Tricky figure numbering
#set figure(
  numbering: n => {
    ([??], [!!], [?!]).at(n - 1)
  },
)
// No prefix
#set ref(supplement: "")

// Justified paragraphs
#set par(justify: true)

// Two-column body
#show: rest => columns(2, rest)

// headcolor
#let headcolor = rgb("004225")

// Run-in sections, like LaTeX \paragraph
#show heading.where(level: 1): it => text(
  size: normalsize,
  weight: "bold",
  fill: headcolor,
  it.body + h(0.67em),
)

// Black raw code
#show raw.where(block: false): it => { it.text }

// Title
#align(
  center,
  link("https://github.com/johanvx/typst-undergradmath")[
    #text(large, headcolor)[*Typst Math for Undergrads*]
  ],
)

// Put this here to avoid affecting the title
#show link: underline

This is a Typst port of _#LaTeX Math for Undergrads_ by Jim Hefferon.
The original version is available at #link("https://gitlab.com/jim.hefferon/undergradmath").

= Meaning of annotations
#figure(
  table(
    columns: (1fr, 2fr),
    [??], [This is unavailable. Last check date is #date.],
  ),
) <unavailable>
#figure(
  table(
    columns: (1fr, 2fr),
    [!!], [Get this in a tricky way. Need a simpler method.],
  ),
) <tricky>
// #figure(
//   table(
//     columns: (1fr, 2fr),
//     [?!], [Don't know how to get this.],
//   )
// ) <noidea>

= Rule One
Any mathematics at all, even a single character, gets a mathematical setting.
Thus, for "the value of $x$ is $7$" enter `the value of $x$ is $7$`.

= Template
Your document should contain at least this.

#grid(
  "",
  ```
  -- document body here --
  ```
)

= Common constructs
#align(
  center,
  table(
    columns: 4,
    align: (right, left, right, left),
    column-gutter: (1em, 1.5em, 1em),
    [$x^2$], [`x^2`],
    [$sqrt(2)$, $root(n, 3)$], [`sqrt(2)`, `root(n, 3)`],
    [$x_(i, j)$], [`x_(i, j)`],
    [$2 / 3$, $2 \/ 3$], [`2 / 3`, `2 \/ 3` or `2 slash 3`], // Maybe use `slash`?
  ),
)

= Calligraphic letters
Use as in `$cal(A)$`.

$ cal(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z) $

Getting script letters is @unavailable.

= Greek
#align(
  center,
  table(
    columns: 4,
    align: (right, left, right, left),
    column-gutter: (1em, 1.5em, 1em),
    [$alpha$], [`alpha`], [$xi$, $Xi$], [`xi`, `Xi`],
    [$beta$], [`beta`], [$omicron$], [`omicron`],
    [$gamma$, $Gamma$], [`gamma`, `Gamma`], [$pi$, $Pi$], [`pi`, `Pi`],
    [$delta$, $Delta$], [`delta`, `Delta`], [$pi.alt$], [`pi.alt`],
    [$epsilon.alt$], [`epsilon.alt`], [$rho$], [`rho`],
    [$epsilon$], [`epsilon`], [$rho.alt$], [`rho.alt`],
    [$zeta$], [`zeta`], [$sigma$, $Sigma$], [`sigma`, `Sigma`],
    [$eta$], [`eta`], [$sigma.alt$], [`sigma.alt`],
    [$theta$, $Theta$], [`theta`, `Theta`], [$tau$], [`tau`],
    [$theta.alt$], [`theta.alt`], [$upsilon$, $Upsilon$], [`upsilon`, `Upsilon`],
    [$iota$], [`iota`], [$phi.alt$, $Phi$], [`phi.alt`, `Phi`],
    [$kappa$], [`kappa`], [$phi$], [`phi`],
    [$lambda$, $Lambda$], [`lambda`, `Lambda`], [$chi$], [`chi`],
    [$mu$], [`mu`], [$psi$, $Psi$], [`psi`, `Psi`],
    [$nu$], [`nu`], [$omega$, $Omega$], [`omega`, `Omega`],
  ),
)

= Sets and logic
#align(
  center,
  table(
    columns: 6,
    align: (right, left, right, left, right, left),
    column-gutter: (1em, 1.5em, 1em, 1.5em, 1em),
    [$union$], [`union`], [$RR$], [`RR`, `bb(R)`], [$forall$], [`forall`],
    [$sect$], [`sect`], [$bb(Z)$], [`ZZ`, `bb(Z)`], [$exists$], [`exists`],
    [$subset$], [`subset`], [$bb(Q)$], [`QQ`, `bb(Q)`], [$not$], [`not`],
    [$subset.eq$], [`subset.eq`], [$bb(N)$], [`NN`, `bb(N)`], [$or$], [`or`],
    [$supset$], [`supset`], [$bb(C)$], [`CC`, `bb(C)`], [$and$], [`and`],
    [$supset.eq$], [`supset.eq`], [$diameter$], [`diameter`], [$tack.r$], [`tack.r`],
    [$in$], [`in`], [$nothing$], [`nothing`], [$models$], [`models`],
    [$in.not$], [`in.not`], [$alef$], [`alef`], [$without$], [`without`],
  ),
)

Negate an operator, as in $subset.not$, with `subset.not`.
Get the set complement $A^(sans(c))$ with `A^(sans(c))` (or $A^(complement)$ with `A^(complement)`, or $overline(A)$ with `overline(A)`).

// https://www.ctan.org/tex-archive/fonts/newcomputermodern
//
// README
//
//     Version 3.93
//
//       Provides access to Russian and Greek guillemotleft and guillemotright
//     using the character variant tables cv3 and cv4 respectively.
//
//       The Math fonts provide the character \varnothing, an alternative to \emptyset,
//     through Character Variant cv01. The fontsetup package provides the option
//     'varnothing' to easily switch to the alternative character.

// https://mirrors.sustech.edu.cn/CTAN/fonts/newcomputermodern/doc/newcm-doc.pdf
// The NewComputerModern FontFamily §13.3
// The Math fonts provide the character \varnothing (⌀, U+2300), as an alternative to \emptyset (a slashed zero), through Character Variant cv01.
// The fontsetup package provides the option ‘varnothing’ to easily switch to the alternative character.

/ Remark: Using `diameter` for `\varnothing` may cause some confusion. However, #LaTeX also uses $diameter$ (`\u{2300}`) instead of $\u{2205}$ (`\u{2205}`), see #link("https://mirrors.sustech.edu.cn/CTAN/fonts/newcomputermodern/doc/newcm-doc.pdf")[newcm $section$13.3].
  Another solution is to use `text(font: "Fira Sans", nothing)`, but the resultant glyph $text(font: "Fira Sans", nothing)$ is subtly different from the widely used one.
  Ultimately, the choice is always *your decision*.

= Decorations
#align(
  center,
  table(
    columns: 6,
    align: (right, left, right, left, right, left),
    column-gutter: (1em, 1.5em, 1em, 1.5em, 1em),
    [$f'$], [`f'`, `f prime`], [$dot(a)$], [`dot(a)`], [$tilde(a)$], [`tilde(a)`],
    [$f prime.double$], [`f prime.double`], [$diaer(a)$], [`diaer(a)`], [$macron(a)$], [`macron(a)`],
    [$Sigma^*$], [`Sigma^*`], [$hat(a)$], [`hat(a)`], [$arrow(a)$], [`arrow(a)`],
  ),
)

If the decorated letter is $i$ or $j$ then some decorations need `dotless.i` and `dotless.j`, as in $arrow(dotless.i)$ with `arrow(dotless.i)`.
Some authors use boldface for vectors: `bold(x)`.

Entering `overline(x + y)` produces $overline(x + y)$, and `hat(x + y)` gives $hat(x + y)$.
Comment on an expression as here (there is also `overbrace(..)`).

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ underbrace(x + y, |A|) $],
    [
      ```
      underbrace(x + y, |A|)
      ```
    ],
  ),
)

= Dots
Use low dots in a list ${0, 1, 2, ...}$, entered as `{0, 1, 2, ...}`.
Use centered dots in a sum or product $1 + dots.h.c + 100$, entered as `1 + dots.h.c + 100`.
You can also get vertical dots `dots.v`, diagonal dots `dots.down` and anti-diagonal dots `dots.up`.

= Roman names
Just type them!

#align(
  center,
  table(
    columns: 6,
    align: (right, left, right, left, right, left),
    column-gutter: (1em, 1.5em, 1em, 1.5em, 1em),
    [$sin$], [`sin`], [$sinh$], [`sinh`], [$arcsin$], [`arcsin`],
    [$cos$], [`cos`], [$cosh$], [`cosh`], [$arccos$], [`arccos`],
    [$tan$], [`tan`], [$tanh$], [`tanh`], [$arctan$], [`arctan`],
    [$sec$], [`sec`], [$coth$], [`coth`], [$min$], [`min`],
    [$csc$], [`csc`], [$det$], [`det`], [$max$], [`max`],
    [$cot$], [`cot`], [$dim$], [`dim`], [$inf$], [`inf`],
    [$exp$], [`exp`], [$ker$], [`ker`], [$sup$], [`sup`],
    [$log$], [`log`], [$deg$], [`deg`], [$liminf$], [`liminf`],
    [$ln$], [`ln`], [$arg$], [`arg`], [$limsup$], [`limsup`],
    [$lg$], [`lg`], [$gcd$], [`gcd`], [$lim$], [`lim`],
  ),
)

= Other symbols
#align(
  center,
  table(
    columns: 6,
    align: (right, left, right, left, right, left),
    column-gutter: (0.5em, 1em, 0.5em, 1em, 0.5em),
    [$<$], [`<`, `lt`], [$angle$], [`angle`], [$dot$], [`dot`],
    [$<=$], [`<=`, `lt.eq`], [$angle.arc$], [`angle.arc`], [$plus.minus$], [`plus.minus`],
    [$>$], [`>`, `gt`], [$ell$], [`ell`], [$minus.plus$], [`minus.plus`],
    [$>=$], [`>=`, `gt.eq`], [$parallel$], [`parallel`], [$times$], [`times`],
    [$!=$], [`!=`, `eq.not`], [$45 degree$], [`45 degree`], [$div$], [`div`],
    [$<<$], [`<<`, `lt.double`], [$tilde.equiv$], [`tilde.equiv`], [$*$], [`*`, `ast`],
    [$>>$], [`>>`, `gt.double`], [$tilde.equiv.not$], [`tilde.equiv.not`], [$divides$], [`divides`],
    [$approx$], [`approx`], [$tilde$], [`tilde`], [$divides.not$], [`divides.not`],
    [$\u{224D}$], [`\u{224D}` @tricky], [$tilde.eq$], [`tilde.eq`], [$n!$], [`n!`],
    [$equiv$], [`equiv`], [$tilde.not$], [`tilde.not`], [$diff$], [`diff`],
    [$prec$], [`prec`], [$plus.circle$], [`plus.circle`], [$nabla$], [`nabla`],
    [$prec.eq$], [`prec.eq`], [$minus.circle$], [`minus.cirle`], [$planck.reduce$], [`planck.reduce`],
    [$succ$], [`succ`], [$dot.circle$], [`dot.circle`], [$compose$], [`compose`],
    [$succ.eq$], [`succ.eq`], [$times.circle$], [`times.circle`], [$star$], [`star`],
    [$prop$], [`prop`], [$\u{2298}$], [`\u{2298}` @tricky], [$sqrt("")$], [`sqrt("")`],
    [$\u{2250}$], [`\u{2250}` @tricky], [$harpoon.tr$], [`harpoon.tr`], [$checkmark$], [`checkmark`],
  ),
)

Use `a divides b` for the divides relation, $a divides b$, and `a divides.not b` for the negation, $a divides.not b$.
Use `|` to get set builder notation ${a in S | a "is odd"}$ with `{a in S | a "is odd"}`.

= Arrows
#align(
  center,
  table(
    columns: 4,
    align: (right, left, right, left),
    column-gutter: (1em, 1.5em, 1em),
    [$->$], [`->`, `arrow.r`], [$|->$], [`|->`, `arrow.r.bar`],
    [$arrow.r.not$], [`arrow.r.not`], [$arrow.r.long.bar$], [`arrow.r.long.bar`],
    [$-->$], [`-->`, `arrow.r.long`], [$<-$], [`<-`, `arrow.l`],
    [$=>$], [`=>`, `arrow.r.double`], [$<->$], [`<->`, `arrow.l.r`],
    [$arrow.r.double.not$], [`arrow.r.double.not`], [$arrow.b$], [`arrow.b`],
    [$==>$], [`==>`, `arrow.r.double.long`], [$arrow.t$], [`arrow.t`],
    [$arrow.squiggly$], [`arrow.squiggly`], [$arrow.t.b$], [`arrow.t.b`],
  ),
)

The right arrows in the first column have matching left arrows, such as `arrow.l.not`, and there are some other matches for down arrows, etc.

= Variable-sized operators
The summation $sum_(j = 0)^3 j^2$ `sum_(j = 0)^3 j^2` and the integral $integral_(x = 0)^3 x^2 dif x$ `integral_(x = 0)^3 x^2 dif x` expand when displayed.

$ sum_(j = 0)^3 j^2 qquad integral_(x = 0)^3 x^2 dif x $

These do the same.

#align(
  center,
  table(
    columns: 4,
    align: (right, left, right, left),
    column-gutter: (1em, 1.5em, 1em),
    row-gutter: 0.5em,
    [$integral$], [`integral`], [$integral.double$], [`integral.double`],
    [$integral.triple$], [`integral.triple`], [$integral.cont$], [`integral.cont`],
    [$union.big$], [`union.big`], [$sect.big$], [`sect.big`],
  ),
)

= Fences
#align(
  center,
  table(
    columns: 6,
    align: (right, left, right, left, right, left),
    column-gutter: (1em, 1.5em, 1em, 1.5em, 1em),
    row-gutter: 0.5em,
    [$()$], [`()`], [$angle.l angle.r$], [`angle.l angle.r`], [$abs("")$], [`abs("")`],
    [$[]$], [`[]`], [$floor("")$], [`floor("")`], [$norm("")$], [`norm("")`],
    [${}$], [`{}`], [$ceil("")$], [`ceil("")`],
  ),
)

Fix the size with the `lr` function.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ lr([sum_(k = 0)^n e^(k^2)], size: #50%) $],
    [
      ```
      lr([sum_(k = 0)^n e^(k^2)], size: #50%)
      ```
    ],
  ),
)

To have them grow with the enclosed formula, also use the `lr` function.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ lr(angle.l i, 2^(2^i) angle.r) $],
    [
      ```
      lr(angle.l i, 2^(2^i) angle.r)
      ```
    ],
  ),
)

Fences scale by default if entered directly as codepoints, and don't scale automatically if entered as symbol notation.

#align(
  center,
  table(
    columns: 2,
    align: (right + horizon, left + horizon),
    column-gutter: 1em,
    [$ (1 / n^(alpha)) $],
    [
      ```
      (1 / n^(alpha))
      ```
    ],

    [$ paren.l 1 / n^(alpha) paren.r $],
    [
      ```
      paren.l 1 / n^(alpha) paren.r
      ```
    ],
  ),
)

The `lr` function also allows to scale unmatched delimiters and one-side fences.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ lr(frac(dif f, dif x) |)_(x_0) $],
    [
      ```
      lr(frac(dif f, dif x) |)_(x_0)
      ```
    ],
  ),
)

= Arrays, Matrices
Get a matrix with the `mat` function. You can pass an array to it.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ mat(a, b; c, d) $],
    [
      ```
      $ mat(a, b; c, d) $
      ```
    ],
  ),
)

In Typst, #link("https://typst.app/docs/reference/types/array")[array] is a sequence of values,
while in #LaTeX, array is a matrix without fences, which is `$mat(delim: #none, ..)$` in Typst.

For the determinant use `|A|`, text operator $det$ `det` or `mat(delim: "|", ..)`.

Definition by cases can be easily obtained with the `cases` function.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [
      $
        f_n = cases(
      a &"if" n = 0,
      r dot f_(n - 1) &"else"
    )
      $
    ],
    [
      ```
      $ f_n = cases(
        a &"if" n = 0,
        r dot f_(n - 1) &"else"
      ) $
      ```
    ],
  ),
)

= Spacing in mathematics
Improve $sqrt(2) x$ to $sqrt(2) thin x$ with a thin space, as in `sqrt(2) thin x`.
Slightly wider are `medium` and `thick` (the three are in ratio $3 : 4 : 5$).
Bigger space is `quad` for $arrow.r quad arrow.l$, which is useful between parts of a display.
Get arbitrary space with the `h` function.
For example, use `#h(2em)` for `\qquad` in #LaTeX and `#h(-0.1667em)` for `\!`.

= Displayed equations
Display equations in a block level using `$ ... $` with at least one space separating the math content and the `$`.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ S = k dot lg W $],
    [
      ```
      $ S = k dot lg W $
      ```
    ],
  ),
)

You can break into multiple lines.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [
      $
        sin(x) = x - x^3 / 3! \
        + x^5 / 5! - dots.h.c
      $
    ],
    [
      ```
      $ sin(x) = x - x^3 / 3! \
          + x^5 / 5! - dots.h.c $
      ```
    ],
  ),
)

Align equations using `&`

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [
      $
        nabla dot bold(D) &= rho \
        nabla dot bold(B) &= 0
      $
    ],
    ```
    $ nabla dot bold(D) &= rho \
      nabla dot bold(B) &= 0 $
    ```,
  ),
)

(the left or right side of an alignment can be empty).
Get a numbered version by `#set math.equation(numbering: ..)`.

= Calculus examples
The last three here are display style.

#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ f: RR -> RR $],
    [
      ```
      f: RR -> RR
      ```
    ],

    [$ "9.8" "m/s"^2 $], [`"9.8" "m/s"^2` @tricky],
    [$ lim_(h->0) (f(x+h)-f(x)) / h $],
    [
      ```
      lim_(h -> 0) (f(x + h) - f(x)) / h
      ```
    ],

    [$ integral x^2 dif x = x^3 \/ 3 + C $],
    [
      ```
      integral x^2 dif x = x^3 \/ 3 + C
      ```
    ],

    [$ nabla = bold(i) dif / (dif x) + bold(j) dif / (dif y) + bold(k) dif / (dif z) $],
    [
      ```
      nabla = bold(i) dif / (dif x) + bold(j) dif / (dif y) + bold(k) dif / (dif z)
      ```
    ],
  ),
)

= Discrete mathematics examples
For modulo, there is a symbol $equiv$ from `equiv` and a text operator $mod$ from `mod`.

For combinations the binomial symbol $binom(n, k)$ is from `binom(n, k)`.
This resizes to be bigger in a display.

For permutations use $n^(underline(r))$ from `n^(underline(r))` (some authors use $P(n, r)$, or $""_n P_r$ from `""_n P_r`).

= Statistics examples
#align(
  center,
  table(
    columns: 2,
    column-gutter: 1em,
    [$ sigma^2 = sqrt(sum(x_i - mu)^2 \/ N) $],
    [
      ```
      sigma^2 = sqrt(sum(x_i - mu)^2 \/ N)
      ```
    ],

    [$ E(X) = mu_X = sum(x_i - P(x_i)) $],
    [
      ```
      E(X) = mu_X = sum(x_i - P(x_i))
      ```
    ],
  ),
)

The probability density of the normal distribution

$ 1 / sqrt(2 sigma^2 pi) e^(- (x - mu)^2 / (2 sigma^2)) $

comes from this.

#align(
  center,
  [
    ```
    1 / sqrt(2 sigma^2 pi)
      e^(- (x - mu)^2 / (2 sigma^2))
    ```
  ],
)

= For more
See also the Typst Documentation at #link("https://typst.app/docs").

#v(1fr)

#block(line(length: 100%, stroke: headcolor) + text(headcolor)[johanvx (https://github.com/johanvx) #h(1fr) #date])
