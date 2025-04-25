
#let template(content) = {
  set text(size: 14pt)
  set par(justify: true, first-line-indent: 2em)

  content
}

#show: template

#show raw: rect

#linebreak()

比起我们最初想要的representation theorem，这里证明的版本多出了一个条件：$η_1$ 和 $η_2$ 之间要存在至少一个relation $R in "Rel"_Σ (η_1, η_2)$。。但是，不是所有 $η_1$ 和 $η_2$ 之间都存在至少一个relation。例如，考虑一个abstract type $α$，它有如下的操作：

$ c: alpha #h(5em) f: alpha arrow "Ans" $

那么，不同的实现可以给 $(f c) : "Ans"$ 赋予不同的值，从而使用这个abstract type的程序就能观测出两个实现的不同了！所以，要求两个实现是被logical relation 连在一起是必要的、也是合理的。

虽然不受限制的representation theorem是不成立的，但是我们可以找到很多具体的例子，它们能够应用 fundamental theorem of logical relation。如果一个签名$Σ$里没有任何操作，那么此时任何两个实现$η_1$、$η_2$ 之间都可以联系起来：取 $ρ(α) = η_1 (α) times η_2 (α)$ 即可。这意味着，由于没有任何操作，α 中的任何值都是无法分辨的。更一般地，如果所有操作都形如$A_1 arrow ... arrow A_n arrow α$，也就是我们利用签名中的操作无法观测一个abstract type的话，那么同样可以证明任何两个实现都是related的，从而证明任何两个实现都无法区分。

```js
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';
const mainContent = 'Hello, typst!';

console.log(await $typst.svg({ mainContent }));
```


比起我们最初想要的representation theorem，这里证明的版本多出了一个条件：$η_1$ 和 $η_2$ 之间要存在至少一个relation $R in "Rel"_Σ (η_1, η_2)$。。但是，不是所有 $η_1$ 和 $η_2$ 之间都存在至少一个relation。例如，考虑一个abstract type $α$，它有如下的操作：

$ c: alpha #h(5em) f: alpha arrow "Ans" $

那么，不同的实现可以给 $(f c) : "Ans"$ 赋予不同的值，从而使用这个abstract type的程序就能观测出两个实现的不同了！所以，要求两个实现是被logical relation 连在一起是必要的、也是合理的。

虽然不受限制的representation theorem是不成立的，但是我们可以找到很多具体的例子，它们能够应用 fundamental theorem of logical relation。如果一个签名$Σ$里没有任何操作，那么此时任何两个实现$η_1$、$η_2$ 之间都可以联系起来：取 $ρ(α) = η_1 (α) times η_2 (α)$ 即可。这意味着，由于没有任何操作，α 中的任何值都是无法分辨的。更一般地，如果所有操作都形如$A_1 arrow ... arrow A_n arrow α$，也就是我们利用签名中的操作无法观测一个abstract type的话，那么同样可以证明任何两个实现都是related的，从而证明任何两个实现都无法区分。

```js
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';
const mainContent = 'Hello, typst!';

console.log(await $typst.svg({ mainContent }));
```

