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

Note that I am only a day old in Rust so there's likely a lot that can be improved in terms of
performance and readability. Please feel free to make suggestions or send PRs.

## Next Steps

Now that we have a decently fast emulator, the next step is to try to solve Enigma!

There's a great [Computerphile video](https://www.youtube.com/watch?v=RzWB5jL5RX0) on this where they use
sentence structure indicators as the objective function. This will be worth a try.