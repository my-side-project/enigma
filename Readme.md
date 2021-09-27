## Enigma Emulator

This project is inspired by a [Numberphile video](https://www.youtube.com/watch?v=G2_Q9FoD-oQ)
with an actual enigma machine!

In order to solve Enigma, I figured I'll need a fast Enigma emulator. I managed to write one
pretty quickly in Python (thanks to this [Wikipedia article](https://en.wikipedia.org/wiki/Enigma_machine)
and this [Java implementation](https://github.com/mikepound/enigma)). However, like most things
Python, it was very slow. I couldn't think of a good way to vectorize the problem and run with Numpy.

That brought me to try Rust which has been on my wishlist for a long time. With a little struggle,
I managed to translate my Python code to Rust. That sped it up by > 9x! Thanks to `cargo-flamegraph`
, I managed to optimize further. Finally, as the `u8` arrays need to be accessed very often for lookups,
I converted them to fixed length arrays instead of `Vec<u8>` as they live on a stack. That made things
50% further faster. At this point, we're able to encrypt 100k 100 character messages (see the `benchmark_test`)
in around 2.8s. These figures are on a MacBook so there's a high variance but the difference is large enough.

| Version                   | Performance |
|---------------------------|-------------|
| Python                    |  65.5s      |
| Rust basic                |  7.5s       |
| Rust after profiling opt. |  4.8s       |
| All vectors to arrays     |  2.8s       |

Note that I am only a day old in Rust atm so there's likely a lot that can be improved in terms of
performance and readability. Please feel free to make suggestions or send PRs.

## Enigma Solver

The package also includes an enigma solver that uses the emulator to make the best guess using the
[Index of Coincidence](https://en.wikipedia.org/wiki/Index_of_coincidence) metric as an objective function.
This method is similar to what's discussed in this [Computerphile video](https://www.youtube.com/watch?v=RzWB5jL5RX0).

The solver currently makes the assumption that we already know the initial positions (in WW2, these were
provided in a separate message). We also assume we know number of rotors and number of wires in the plugboard.

While it tends to be fairly accurate in my testing, it's a bit slow and does not use multiple CPUs. Even single threaded,
there's some opportunity to improve performance by optimizing the Rust code (again, I am now two days old in Rust).

I have added some example ciphers to solve in the repo for testing and continued benchmarking:

```
➜  solver git:(main) ✗ cargo run -- -c "easy_three_rings_no_wires.txt" -r 3 -w 0 -s
   Compiling solver v0.1.0 (/Users/mihirsathe/Documents/GitHub/enigma/solver)
    Finished dev [unoptimized + debuginfo] target(s) in 1.55s
     Running `target/debug/solver -c easy_three_rings_no_wires.txt -r 3 -w 0 -s`
Input text: URPLQGZOHIDTBQMYIMTGUQYOFLKHSNODZZAJIZXEYTYHOZOBVTQAWFLGRJGKISZUMWJFLGSJOBBJFRFBKIUIKEWUREGKXXFHZIROYIYLITJNLIFWIQGOSDQCXVPWABADMLUJZMEIKQMIMYYCRKCCJMDESPVMJJBEUGHZNHTVDBPYORUUERQSTSXYOOJFHZWSTJWXLMJJJMWQIDOYWPNGPPXALFTRKRAOFZEKCV
Initial IOC: 977
Best parameters: rotors: [1, 0, 2], settings: [2, 5, 2], score: 1571, plugboard: {}
Best guess: ITWASABRILLIANTAFTERHOONINLATESPRINGANDVINDOBOUAWASTAKINGITSPLEASUREJOYOJSLYOUTOFDOORSASISITSWONTTYEMANYPARKSANDGARDENSOFTHECITYWERECROWDEDWITHHOLIDAYNAKERSINEVERYVARIETYOFNATIUNALCOSTUMEANDSPEAKINGTHETMNGUESOFALLTHEEARTHANDINTHMB
Deciphered in 9915ms
```

```
➜  solver git:(main) ✗ cargo run -- -c "medium_three_rings_five_wires.txt" -r 3 -w 3 -s
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/solver -c medium_three_rings_five_wires.txt -r 3 -w 3 -s`
Input text: URMLQGZWHIFRBQXYNEYLUQYODLKHWBOFSZAJIZKWYRLHOZOBZRQOEDLGTJAKISFUMECRLGSJOBBJDTJBZTUIUWEUTWGDXSDHIYTIYIYLIRBNLIDSIQISDFQCVMPWABNFOBYNZQWROFTDMHYCTKCNJMFWSSLMJJZWPIHZNJRLIBPBOTAUWTQSRSXFOORDHGESOJEXLMFJCOEQIFTYEGNGP
Initial IOC: 1023
Best parameters: rotors: [1, 0, 2], settings: [2, 5, 2], score: 1558, plugboard: {'R': 'T', 'E': 'W', 'D': 'F'}
Best guess: ITWASABRILLIANTAFTERHOONINLATESPRINGANDVINDOBOUAWASTAKINGITSPLEASUREJOYOJSLYOUTOFDOORSASISITSWONTTYEMANYPARKSANDGARDENSOFTHECITYWERECROWDEDWITHHOLIDAYNAKERSINEVERYVARIETYOFNATIUNALCOSTUMEANDSPEAKINGTHETMNGUESOFALL
Deciphered in 9582ms
```

```
➜  solver git:(main) ✗ cargo run -- -c "hard_five_rings_five_wires.txt" -r 5 -w 5 -s
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/solver -c hard_five_rings_five_wires.txt -r 5 -w 5 -s`
Input text: HJXPPDESPDHVDMYFKLCMFSHGFDBIPFYTWNDDHEJLHJEHMRLXAXBBFTDXMDOJTNNOVWXKVZPVRGOTPAKEKLIXIHBJSHMFAZDVKOIOXFUDFDJALTKPYXNYFAKNPKQBEAULBXLMVWACIIHXXJDAJOXCPNXFJLBUKDFHYAIFJOYCOINDOLOQMYSAJSEJISXHOZPKSFGHJLXIFXPSNWSTCPSWHEGQFCZVUBHRLELYAXHRRGNTAIEYTYRQJYEZWKFSWVBDSIIXXYAVVBHOQZTCTZBOKHBMFKYOPAHLJGAQSHODPCPFKTFOPKBPVUVZJLLZNRCYORPGFYCDKZNOQSJIDJYJSESIFBOPMYHMFCWWZZGMFFWRIUGPAIJOPAWQYANZVXJHUAKUNRCYXNIWIJTVXRXCPQJDSYPGTHSWLBXLSNWFSNCXRQLUSAANGTJAZGDWYEZEXEOKDELNKJNSHSBVIIZRDJANYUDBOBFHHFYXVVPRZWXKI
Initial IOC: 1026
Best parameters: rotors: [4, 0, 2], settings: [2, 5, 2], score: 1613, plugboard: {'K': 'L', 'D': 'F', 'O': 'P', 'R': 'T', 'E': 'W'}
Best guess: ITWASABRILLIANTAFTERZOONINLATESPRINGANDVINDOBOCAWASTAKINGITSPLEASUREJOYOYSLYOUTOFDOORSASISITSWONTTKEMANYPARKSANDGARDENSOFTHEVITYWERECROWDEDWITHHOLIDAYUAKERSINEVERYVARIETYOFNATICNALCOSTUMEANDSPEAKINGTHETHNGUESOFALLTHEEARTHANDINTHPBOULEVARDSOFTHERINGSTRASSIAWELLDRESSEDTHRONGMADETHENAVEMENTSALMOSTIMPASSABLETSEREWASNOTAVACANTSEATTOBEFYUNDATTHEROWSOFTABLESOUTSIQEEACHCAFWHERESTRANGEANDWOFHIHINHVYUTKLTLOCXJXAWBKKLJLMEDINVASTQUANTITIESBUTWIAHADELIBERATIONTHATIMPLIEDRHEPOSSESSIONOFUNLIMITEDLEASURE
Deciphered in 199221ms
```

Note: For harder settings with many wires in the plugboard, use at least 500 character text as
our function has a hard time identifying language features through frequency analysis when
the text is too small.

## Next Steps

One thing that lowers our accuracy with wires is that we commit too quickly and only choose the
best scoring solution every time. Using top n solutions (similar to when finding ring combinations)
will help fix that prroblem.

There's probably some combinatorics to be used to develop good heuristics for when to stop.
For example, finding common factors among similarly well or poorly performing solutions.

It will be interesting to see how much performance boost we can get (at how much accuracy drop)
by using random experiments instead of exhaustive search.

While it's quite remarkable how well IoC works (considering how simple it is), we should explore
more sophisticated metrics that can especially do a better job with shorter texts. One idea
(potentially next weekend project?) is to develop an RNN to guess how close to an actual sentence
a piece of text is. It's easy to generate training data for this. Would be interesting to see how 
generalizable it is.