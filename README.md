# stealer
bare-bones parallel iterator

Rayon is fantastic.
It is feature rich and a beauty to use.

Stealer throws this all out because I wrote it in an afternoon and am lazy.
That being said, for large numbers of iterations and result aggrigation, It is extrememly fast.
For a concurrent fizzbuzz of 0 to 100000000 it was 5.99x the speed of rayon.