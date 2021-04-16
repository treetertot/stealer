# stealer
bare-bones parallel iterator

Note: this project is dead. Rayon released an update that makes it faster than stealer a while ago.
I'm still not sure how this ever ended up faster lol.

Rayon is fantastic.
It is feature rich and a beauty to use.

Stealer throws this all out because I wrote it in an afternoon and am lazy.
That being said, for large numbers of iterations and result aggrigation, It is extrememly fast.
For a concurrent fizzbuzz of 0 to 100000000 it was 5.99x the speed of rayon.

The api consists of a function called run that takes an ExactSizeIterator and the closure to run.
That's it.

UNTIL NOW!!!
Now we have ranges, which are way less versatile, but should be even faster
