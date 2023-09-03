
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set heading(numbering: "1.", supplement: [Chapter])
#set math.equation(numbering: "(1)", supplement: [Eq.])

= Intro
#figure(
  image("/assets/files/cylinder.svg", height: 1cm),
  caption: [A cylinder.],
  supplement: "Fig",
) <fig1>

#figure(
  image("/assets/files/tiger.jpg", height: 1cm),
  caption: [A tiger.],
  supplement: "Tig",
) <fig2>

$ A = 1 $ <eq1>

#set math.equation(supplement: none)
$ A = 1 $ <eq2>

@fig1, @fig2, @eq1, (@eq2)

#set ref(supplement: none)
@fig1, @fig2, @eq1, @eq2
