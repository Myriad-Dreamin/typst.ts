// https://github.com/werifu/HUST-typst-template/blob/1ef0a73007b0827e04dbe00b0849890a94eace42/sample.typ

#let heiti = ("Times New Roman", "Heiti SC", "Heiti TC", "SimHei")
#let songti = ("Times New Roman", "Songti SC", "Songti TC", "SimSun")
#let zhongsong = ("Times New Roman", "Songti SC")

#let bib_cite(..name) = {
  super(cite(..name))
}

#let indent() = {
  box(width: 2em)
}

#let indent_par(body) = {
  box(width: 1.8em)
  body
}

#let equation_num(_) = {
  locate(loc => {
    let chapt = counter(heading).at(loc).at(0)
    let c = counter("equation-chapter" + str(chapt))
    let n = c.at(loc).at(0)
    "(" + str(chapt) + "-" + str(n + 1) + ")"
  })
}

#let table_num(_) = {
  locate(loc => {
    let chapt = counter(heading).at(loc).at(0)
    let c = counter("table-chapter" + str(chapt))
    let n = c.at(loc).at(0)
    str(chapt) + "-" + str(n + 1)
  })
}

#let image_num(_) = {
  locate(loc => {
    let chapt = counter(heading).at(loc).at(0)
    let c = counter("image-chapter" + str(chapt))
    let n = c.at(loc).at(0)
    str(chapt) + "-" + str(n + 1)
  })
}


#let equation(equation, caption: "") = {
  figure(
    equation,
    caption: caption,
    supplement: [公式],
    numbering: equation_num,
    kind: "equation",
  )
}

#let tbl(tbl, caption: "") = {
  figure(
    tbl,
    caption: caption,
    supplement: [表],
    numbering: table_num,
    kind: "table",
  )
}

#let img(img, caption: "") = {
  figure(
    img,
    caption: caption,
    supplement: [图],
    numbering: image_num,
    kind: "image",
  )
}


#let empty_par() = {
  v(-1em)
  box()
}

// inspired from https://github.com/lucifer1004/pkuthss-typst.git
#let chinese_outline() = {
  align(center)[
    #text(font: heiti, size: 18pt, "目　　录")
  ]

  set text(font: songti, size: 12pt)
  // 临时取消目录的首行缩进
  set par(leading: 1.24em, first-line-indent: 0pt)
  locate(loc => {
    let elements = query(heading.where(outlined: true), loc)
    for el in elements {
      // 是否有 el 位于前面，前面的目录中用拉丁数字，后面的用阿拉伯数字
      let before_toc = query(heading.where(outlined: true).before(loc), loc).find((one) => {one.body == el.body}) != none
      let page_num = if before_toc {
        numbering("I", counter(page).at(el.location()).first())
      } else {
        counter(page).at(el.location()).first()
      }

      link(el.location())[#{
        // acknoledgement has no numbering
        let chapt_num = if el.numbering != none {
          numbering(el.numbering, ..counter(heading).at(el.location()))
        } else {none}

        if el.level == 1 {
          set text(weight: "bold")
          if chapt_num == none {} else {
            chapt_num
            "　　"
          }
          el.body
        } else {
          chapt_num
          "　"
          el.body
        }
      }]

      // 填充 ......
      box(width: 1fr, h(0.5em) + box(width: 1fr, repeat[.]) + h(0.5em))
      [#page_num]
      linebreak()
    }
  })
}

// 原创性声明和授权书
#let declaration() = {
  set text(font: songti, 12pt)

  v(3em)
  align(center)[
    #text(font: heiti, size: 18pt)[
      学位论文原创性声明
    ]
  ]
  text(font: songti, size: 12pt)[
    #set par(justify: false, leading: 1.24em, first-line-indent: 2em)
    本人郑重声明：所呈交的论文是本人在导师的指导下独立进行研究所取得的 研究成果。除了文中特别加以标注引用的内容外，本论文不包括任何其他个人或集体已经发表或撰写的成果作品。本人完全意识到本声明的法律后果由本人承担。
  ]
  v(2em)
  align(right)[
    #text("作者签名：　　　　　　　年　　月　　日")
  ]
  
  v(6em)
  align(center)[
    #text(font: heiti, size: 18pt)[
      学位论文版权使用授权书
    ]
  ]
  text(font: songti, size: 12pt)[
    #set par(justify: false, leading: 1.24em, first-line-indent: 2em)
    本学位论文作者完全了解学校有关保障、使用学位论文的规定，同意学校保留并向有关学位论文管理部门或机构送交论文的复印件和电子版，允许论文被查阅和借阅。本人授权省级优秀学士论文评选机构将本学位论文的全部或部分内容编入有关数据进行检索，可以采用影印、缩印或扫描等复制手段保存和汇编本学位论文。

    学位论文属于 1、保密 □，在#h(3em)年解密后适用本授权书。

    #h(6.3em) 2、不保密 □

    #h(6.3em)请在以上相应方框内打 “√”
  ]

  v(3em)
  align(right)[
    #text("作者签名：　　　　　　　年　　月　　日")
  ]

  align(right)[
    #text("导师签名：　　　　　　　年　　月　　日")
  ]
}

// 参考文献
#let references(path) = {
  // 这个取消目录里的 numbering
  set heading(level: 1, numbering: none)

  set par(justify: false, leading: 1.24em, first-line-indent: 2em)

  bibliography(path, title:"参考文献")
}


// 致谢，请手动调用
#let acknowledgement(body) = {
  // 这个取消目录里的 numbering
  set heading(level: 1, numbering: none)
  show <_thx>: {
    // 这个取消展示时的 numbering
    set heading(level: 1, numbering: none)
    set align(center)
    set text(weight: "bold", font: heiti, size: 18pt)

    "致　　谢"
  } + empty_par()

  
  [= 致谢 <_thx>]

  body
}

// 中文摘要
#let zh_abstract_page(abstract, keywords: ()) = {
  set heading(level: 1, numbering: none)
  show <_zh_abstract_>: {
    align(center)[
      #text(font: heiti, size: 18pt, "摘　　要")
    ]
  }
  [= 摘要 <_zh_abstract_>]

  set text(font: songti, size: 12pt)

  abstract
  par(first-line-indent: 0em)[
    #text(weight: "bold", font: heiti, size: 12pt)[
      关键词：
      #keywords.join("；")
    ]
  ]
}

// 英文摘要
#let en_abstract_page(abstract, keywords: ()) = {
  set heading(level: 1, numbering: none)
  show <_en_abstract_>: {
    align(center)[
      #text(font: heiti, size: 18pt, "Abstract")
    ]
  }
  [= Abstract <_en_abstract_>]

  set text(font: songti, size: 12pt)

  abstract
  par(first-line-indent: 0em)[
    #text(weight: "bold", font: heiti, size: 12pt)[
      Key Words: 
      #keywords.join("; ")
    ]
  ]
}

#let project(
  title: "",
  abstract_zh: [],
  abstract_en: [],
  keywords_zh: (),
  keywords_en: (),
  school: "",
  author: "",
  id: "",
  mentor: "",
  class: "",
  date: (1926, 8, 17),
  body,
) = {
  // 引用的时候，图表公式等的 numbering 会有错误，所以用引用 element 手动查
  show ref: it => {
    if it.element != none and it.element.func() == figure {
      let el = it.element
      let loc = el.location()
      let chapt = counter(heading).at(loc).at(0)

      // 自动跳转
      link(loc)[#if el.kind == "image" or el.kind == "table" {
          // 每章有独立的计数器
          let num = counter(el.kind + "-chapter" + str(chapt)).at(loc).at(0) + 1
          it.element.supplement
          " "
          str(chapt)
          "-"
          str(num)
        } else if el.kind == "equation" {
          // 公式有 '(' ')'
          let num = counter(el.kind + "-chapter" + str(chapt)).at(loc).at(0) + 1
          it.element.supplement
          " ("
          str(chapt)
          "-"
          str(num)
          ")"
        } else {
          it
        }
      ]
    } else {
      it
    }
  }

  // 图表公式的排版
  show figure: it => {
    set align(center)
    if it.kind == "image" {
      set text(font: heiti, size: 12pt)
      it.body
      it.supplement
      " " + it.counter.display(it.numbering)
      "　" + it.caption
      locate(loc => {
        let chapt = counter(heading).at(loc).at(0)
        let c = counter("image-chapter" + str(chapt))
        c.step()
      })
    } else if it.kind == "table" {
      set text(font: heiti, size: 12pt)
      it.supplement
      " " + it.counter.display(it.numbering)
      "　" + it.caption
      set text(font: songti, size: 10.5pt)
      it.body
      locate(loc => {
        let chapt = counter(heading).at(loc).at(0)
        let c = counter("table-chapter" + str(chapt))
        c.step()
      })
    } else if it.kind == "equation" {
      // 通过大比例来达到中间和靠右的排布
      grid(
        columns: (20fr, 1fr),
        it.body,
        align(center + horizon, 
          it.counter.display(it.numbering)
        )
      )
      locate(loc => {
        let chapt = counter(heading).at(loc).at(0)
        let c = counter("equation-chapter" + str(chapt))
        c.step()
      })
    } else {
      it
    }
  }
  set page(paper: "a4", margin: (
    top: 2.5cm,
    bottom: 2.5cm,
    left: 3cm,
    right: 3cm
  ))

  // 封面
  align(center)[
    // hust logo
    #v(30pt)

    #image("avatar-min-min.jpeg", width: 55%)

    #v(50pt)

    #text(
      size: 36pt,
      font: zhongsong,
      weight: "bold"
    )[本科生毕业设计(论文)]

    #v(40pt)

    #text(
      font: heiti,
      size: 22pt,
    )[
      #title
    ]

    #v(100pt)

    #let info_value(body) = {
      rect(
        width: 100%,
        inset: 2pt,
        stroke: (
          bottom: 1pt + black
        ),
        text(
          font: zhongsong,
          size: 16pt,
          bottom-edge: "descender"
        )[
          #body
        ]
      ) 
    }
    
    #let info_key(body) = {
      rect(width: 100%, inset: 2pt, 
       stroke: none,
       text(
        font: zhongsong,
        size: 16pt,
        body
      ))
    }

    #grid(
      columns: (70pt, 180pt),
      rows: (40pt, 40pt),
      gutter: 3pt,
      info_key("院　　系"),
      info_value(school),
      info_key("专业班级"),
      info_value(class),
      info_key("姓　　名"),
      info_value(author),
      info_key("学　　号"),
      info_value(id),
      info_key("指导教师"),
      info_value(mentor),
    )

    #v(40pt)
    #text(
      font: zhongsong,
      size: 16pt,
    )[
      #date.at(0) 年 #date.at(1) 月 #date.at(2) 日
    ]
  ]
  
  counter(page).update(0)
  // 页眉
  set page(
    header: {
      set text(font: songti, 10pt, baseline: 8pt, spacing: 3pt)
      set align(center)
      [华 中 科 技 大 学 毕 业 设 计 (论 文)]
      line(length: 100%, stroke: 0.7pt)
    }
  )

  // 页脚
  // 封面不算页数
  set page(
    footer: {
      set align(center)
      
      grid(
        columns: (5fr, 1fr, 5fr),
        line(length: 100%, stroke: 0.7pt),
        text(font: songti, 10pt, baseline: -3pt, 
          counter(page).display("I")
        ),
        line(length: 100%, stroke: 0.7pt)
      )
    }
  )

  set text(font: songti, 12pt)
  set par(justify: true, leading: 1.24em, first-line-indent: 2em)
  show par: set block(spacing: 1.24em)

  set heading(numbering: (..nums) => {
    nums.pos().map(str).join(".") + "　"
  })
  show heading.where(level: 1): it => {
    set align(center)
    set text(weight: "bold", font: heiti, size: 18pt)
    set block(spacing: 1.5em)
    it
  }
  show heading.where(level: 2): it => {
    set text(weight: "bold", font: heiti, size: 14pt)
    set block(above: 1.5em, below: 1.5em)
    it
  }

  // 首段不缩进，手动加上 box
  show heading: it => {
    set text(weight: "bold", font: heiti, size: 12pt)
    set block(above: 1.5em, below: 1.5em)
    it
  } + empty_par()

  // 原创性声明
  declaration()

  pagebreak()
  counter(page).update(1)

  // 摘要
  zh_abstract_page(abstract_zh, keywords: keywords_zh)

  pagebreak()

  // abstract
  en_abstract_page(abstract_en, keywords: keywords_en)

  pagebreak()

  // 目录
  chinese_outline()

  // 正文的页脚
  
  set page(
    footer: {
      set align(center)
      
      grid(
        columns: (5fr, 1fr, 5fr),
        line(length: 100%, stroke: 0.7pt),
        text(font: songti, 10pt, baseline: -3pt, 
          counter(page).display("1")
        ),
        line(length: 100%, stroke: 0.7pt)
      )
    }
  )


  counter(page).update(1)

  // 代码块(TODO: 加入行数)
  show raw: it => {
    set text(font: songti, 12pt)
    set block(inset: 5pt, fill: rgb(217, 217, 217, 1), width: 100%)
    it
  }

  body
}

// 三线表
#let tlt_header(content) = {
  set align(center)
  rect(
    width: 100%,
    stroke: (bottom: 1pt),
    [#content],
  )
}

#let tlt_cell(content) = {
  set align(center)
  rect(
    width: 100%,
    stroke: none,
    [#content]
  )
}

#let tlt_row(r) = {
  (..r.map(tlt_cell).flatten())
}

#let three_line_table(values) = {
  rect(
    stroke: (bottom: 1pt, top: 1pt),
    inset: 0pt,
    outset: 0pt,
    grid(
      columns: (auto),
      rows: (auto),
      // table title
      grid(
        columns: values.at(0).len(),
        ..values.at(0).map(tlt_header).flatten()
      ),

      grid(
        columns: values.at(0).len(),
        ..values.slice(1).map(tlt_row).flatten()
      ),
    )
  )
}

#show: project.with(
  title: "基于 ChatGPT 的狗屁通文章生成器",
  author: "作者",
  abstract_zh: [
  先帝创业未半而中道崩殂，今天下三分，益州疲弊，此诚危急存亡之秋也。然侍卫之臣不懈于内，忠志之士忘身于外者，盖追先帝之殊遇，欲报之于陛下也。诚宜开张圣听，以光先帝遗德，恢弘志士之气，不宜妄自菲薄，引喻失义，以塞忠谏之路也。

  宫中府中，俱为一体；陟罚臧否，不宜异同。若有作奸犯科及为忠善者，宜付有司论其刑赏，以昭陛下平明之理，不宜偏私，使内外异法也。
  ],
  abstract_en: [
  The founding emperor passed away before his endeavor was half completed, and now the empire is divided into three parts. Yizhou is exhausted and in decline, and this is truly a critical moment of survival or destruction. However, the palace guards are tirelessly serving within, and loyal subjects are sacrificing themselves outside, all in order to repay the late emperor's kindness and show loyalty to the current emperor. It is appropriate to listen to wise advice, to honor the late emperor's virtues, to inspire the courage of loyal subjects, and not to belittle oneself or distort the truth, in order to keep the path of loyal counsel open. 

  The palace and government are one entity, and punishments should be consistent. If there are those who commit crimes or show loyalty and virtue, they should be judged by the legal system to demonstrate your fairness as emperor, rather than showing partiality that would create different laws for those inside and outside the palace.
  ],
  keywords_zh: ("关键词1", "关键词2", "关键词3"),
  keywords_en: ("Keyword 1", "Keyword 2", "Keyword 3"),
  school: "复制粘贴写报告学院",
  id: "U000114514",
  mentor: "你的老板",
  class: "XXXX 专业 0000 班",
  date: (1926, 8, 17)
)


= 绪论

== typst 介绍


typst 是最新最热的标记文本语言，定位与 LaTeX 类似，具有极强的排版能力，通过一定的语法写文档，然后生成 pdf 文件。与 LaTeX 相比有以下的优势：


1. 编译巨快：因为提供增量编译的功能所以在修改后基本能在一秒内编译出 pdf 文件，typst 提供了监听修改自动编译的功能，可以像 Markdown 一样边写边看效果。

2. 环境搭建简单：原生支持中日韩等非拉丁语言，不用再大量折腾字符兼容问题以及下载好几个 G 的环境。只需要下载命令行程序就能开始编译生成 pdf。

3. 语法友好：对于普通的排版需求，上手难度跟 Markdown 相当，同时文本源码阅读性高：不会再充斥一堆反斜杠跟花括号

个人观点：跟 Markdown 一样好用，跟 LaTeX 一样强大


#img(
  image("avatar-min-min.jpeg", height: 20%),
  caption: "我的 image 实例 0",
)

== 基本语法

=== 代码执行

正文可以像前面那样直接写出来，隔行相当于分段。

个人理解：typst 有两种环境，代码和内容，在代码的环境中会按代码去执行，在内容环境中会解析成普通的文本，代码环境用{}表示，内容环境用[]表示，在 content 中以 \# 开头来接上一段代码，比如\#set rule，而在花括号包裹的块中调用代码就不需要 \#。


=== 标题

类似 Markdown 里用 \# 表示标题，typst 里用 = 表示标题，一级标题用一个 =，二级标题用两个 =，以此类推。


间距、字体等我都排版好了。

\#pagebreak() 函数相当于分页符，在华科的要求里，第一集标题应当分页，请手动。

#pagebreak()

= 本模板相关封装

== 图片

除了排版，这个模板主要改进了图片、表格、公式的引用，编号以第x章来计算。


图用下面的方式来引用，img 是对 image 的包装，caption 是图片的标题。

#img(
  image("avatar-min-min.jpeg", height: 20%),
  caption: "我的 image 实例 1",
) <img1>

引用的话就在 img 后加上标签<label>，使用 at 来引用即可，比如@img2 就是这么引用的。

#img(
  image("avatar-min-min.jpeg", height: 20%),
  caption: "我的 image 实例 2",
) <img2>

引用 2-1: @img1

== 表格

表格跟图片差不多，但是表格的输入要复杂一点，建议去 typst 官网学习一下，自由度特别高，定制化很强。

看看@tbl1，tbl 也是接收两个参数，一个是 table 本身，一个是标题，table 里的参数，没有字段的一律是单元格里的内容（每一个被[]）包起来的内容，在 align 为水平时横向排列，排完换行。

#tbl(
  table(
    columns: (100pt, 100pt, 100pt),
    inset: 10pt,
    stroke: 0.7pt,
    align: horizon,
    [], [*Area*], [*Parameters*],
    image("avatar-min-min.jpeg", height: 10%),
    $ pi h (D^2 - d^2) / 4 $,
    [
      $h$: height \
      $D$: outer radius \
      $d$: inner radius
    ],
    image("avatar-min-min.jpeg", height: 10%),
    $ sqrt(2) / 12 a^3 $,
    [$a$: 边长]
  ),
  caption: "芝士样表"
) <tbl1>



#tbl(
  three_line_table(
    (
    ("Country List", "Country List", "Country List"),
    ("Country Name or Area Name", "ISO ALPHA Code", "ISO ALPHA"),
    ("Afghanistan", "AF", "AFT"),
    ("Aland Islands", "AX", "ALA"),
    ("Albania", "AL", "ALB"),
    ("Algeria", "DZ", "DZA"),
    ("American Samoa", "AS", "ASM"),
    ("Andorra", "AD", "AND"),
    ("Angola", "AP", "AGO"),
  )),
  caption: "三线表示例"
)

#tbl(
  three_line_table(
    (
    ("Country List", "Country List", "Country List", "Country List"),
    ("Country Name or Area Name", "ISO ALPHA 2 Code", "ISO ALPHA 3", "8"),
    ("Afghanistan", "AF", "AFT", "7"),
    ("Aland Islands", "AX", "ALA", "6"),
    ("Albania", "AL", "ALB", "5"),
    ("Algeria", "DZ", "DZA", "4"),
    ("American Samoa", "AS", "ASM", "3"),
    ("Andorra", "AD", "AND", "2"),
    ("Angola", "AP", "AGO", "1"),
  )),
  caption: "三线表示例2"
)




== 公式

公式用两个\$包裹，但是语法跟 LaTeX 并不一样，如果有大量公式需求那先建议看官网教程，不过typst还比较早期，不排除以后会加入兼容语法的可能。

为了给公式编号，也依然有包装，使用 equation 里包公式的方式实现，比如：


#equation(
  $ A = pi r^2 $,
) <eq1>

根据@eq1 ，推断出@eq2

#equation(
  $ x < y => x gt.eq.not y $,
) <eq2>

然后也有多行的如@eq3，标签名字可以自定义

#equation(
  $ sum_(k=0)^n k
    &< 1 + ... + n \
    &= (n(n+1)) / 2 $,
) <eq3>


如果不想编号就直接用\$即可


$ x < y => x gt.eq.not y $

== 列表

- 无序列表1: 1

- 无序列表2: 2

#indent()列表后的正文，应当有缩进。这里使用一个 indent_par 函数来手动生成段落缩进，在目前的 typst 设计里，按英文排版的习惯，连续段落里的第一段是不会缩进的，也包括各种列表。

1. 有序列表1
2. 有序列表2

列表后的正文，应当有缩进，但是这里没有，请自己在段首加上\#indent()，也可以用 \#indent_par()[] 包裹整段

想自己定义可以自己set numbering，建议用 \#[] 包起来保证只在该作用域内生效：

#[
#set enum(numbering: "1.a)")
+ 自定义列表1
  + 自定义列表1.1
+ 自定义列表2
  + 自定义列表2.1
]

#pagebreak()
= 其他说明

== 文献引用

引用支持 LaTeX Bib 的格式，也支持更简单好看的 yml 来配置，在引用时使用#bib_cite("harry", "某本参考书")以获得右上的引用标注。


记得在最后加入\#references("xxxref.yml")函数的调用来生成参考文献。

*尚不支持国标，自定义参考文献功能官方正在开发中*

== 致谢部分

致谢部分在最后加入\#acknowledgement()函数的调用来生成。


== 模板相关

这个模板应该还不是完全版，可能不能覆盖到华科论文的所有case要求，如果遇到特殊 case 请提交 issue 说明，或者也可以直接提 pull request

同时，由于 typst 太新了，2023年4月初刚发布0.1.0版本，可能会有大的break change发生，模板会做相应修改。

主要排版数据参考来自 https://github.com/zfengg/HUSTtex 同时有一定肉眼排版成分，所以有可能不完全符合华科排版要求，如果遇到不对的间距、字体等请提交 issue 说明。

=== 数据相关

所有拉丁字母均为 Times New Roman，大小与汉字相同

正文为宋体12pt，即小四

图表标题为黑体12pt

表格单元格内容宋体10.5pt，即五号

一级标题黑体18pt加粗，即小二

二级标题14pt加粗，即四号

正文行间距1.5em

a4纸，上下空2.5cm，左右空3cm

#pagebreak()
= 这是一章占位的

== 占位的二级标题 1

== 占位的二级标题 2

== 占位的二级标题 3

== 占位的二级标题 4

=== 占位的三级标题 1

=== 占位的三级标题 2

==== 占位的四级标题 1

==== 占位的四级标题 2

== 占位的二级标题 5

== 占位的二级标题 6



#pagebreak()
#acknowledgement()[
  
完成本篇论文之际，我要向许多人表达我的感激之情。

首先，我要感谢我的指导教师，他/她对本文提供的宝贵建议和指导。所有这些支持和指导都是无私的，而且让我受益匪浅。

其次，我还要感谢我的家人和朋友们，他们一直以来都是我的支持和鼓励者。他们从未停止鼓舞我，勉励我继续前行，感谢你们一直在我身边，给我幸福和力量。

此外，我还要感谢我的同学们，大家一起度过了很长时间的学习时光，互相支持和鼓励，共同进步。因为有你们的支持，我才能不断地成长、进步。

最后，我想感谢笔者各位，你们的阅读和评价对我非常重要，这也让我意识到了自己写作方面的不足，同时更加明白了自己的研究方向。谢谢大家！

再次向所有支持和鼓励我的人表达我的谢意和感激之情。

本致谢生成自 ChatGPT。
]

#pagebreak()
#references("./ref.yml")
// 想用 TeX Bib 可以按下面这样引用
// #references("./ref.bib")