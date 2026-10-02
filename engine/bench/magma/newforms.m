// Rational newforms of weight 2 for levels N in [a, b] with Magma:
// magma -b a:=11 b:=200 [k:=16] newforms.m
// Levels a, a+k, a+2k, ... <= b (k defaults to 1; run k processes with
// a, a+1, ..., a+k-1 to use k cores).  Prints one line per level (N, number
// of rational newforms, seconds) and a total.
a := StringToInteger(a); b := StringToInteger(b);
k := assigned k select StringToInteger(k) else 1;
total := 0; t0 := Cputime();
for N in [a..b by k] do
    t := Cputime();
    M := ModularSymbols(N, 2, +1);
    S := NewSubspace(CuspidalSubspace(M));
    D := NewformDecomposition(S);
    rat := [A : A in D | Dimension(A) eq 1];
    aps := [[Coefficient(qEigenform(A, 100), p) : p in PrimesUpTo(97) | N mod p ne 0] : A in rat];
    total +:= #rat;
    printf "%o %o %.3o\n", N, #rat, Cputime(t);
end for;
printf "levels %o..%o: %o rational newforms, %.2o s CPU\n", a, b, total, Cputime(t0);
quit;
