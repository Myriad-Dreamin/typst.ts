
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test mixtures of math kerning.
#show math.equation: set text(font: "STIX Two Math")

$ x_1^i x_2^lambda x_2^(2alpha) x_2^(k+1) x_2^(-p_(-1)) x_j^gamma \
  f_2^2 v_0^2  z_0^2 beta_s^2 xi_i^k J_1^2 N_(k y)^(-1) V_pi^x \
  attach(J, tl: 1, br: i) attach(P, tl: i, br: 2) B_i_0 phi.alt_i_(n-1)
  attach(A, tr: x, bl: x, br: x, tl: x) attach(F, tl: i, tr: f) \
  attach(cal(A), tl: 2, bl: o) attach(cal(J), bl: l, br: A)
  attach(cal(y), tr: p, bl: n t) attach(cal(O), tl: 16, tr: +, br: sigma)
  attach(italic(Upsilon), tr: s, br: Psi, bl: d) $