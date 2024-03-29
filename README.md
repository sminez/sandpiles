Sandpiles :: Fractal Generation And Visualisation
=================================================

![Sandpile](index.png)

Inspired by the [Numberphile video](https://www.youtube.com/watch?v=1MtEUErz7Gg)
on sandpile fractals, I wrote a small python (later go) program to generate the
fractal shown towards the end of the video. The next obvious step was to try new
toppling patterns and see what they produced!

I was able to get up to seed values of ~2^24 with my original algorithm but more
than that started to take days to complete runs. The original algorithm made a
crude estimate for the upper bound of the grid size based on the seed value and
then iterated over a 2D array to generate the fractals. Given that for a large
part of the computation, the majority of cells are not touched (if ever) this
wasn't very efficient.

The new algorithm uses a hash map for toppling which is then mapped to a 2D
array once we reach a steady state. This also allows for arbitrary co-ordinate
systems and grid layouts to be used which is what I'm exploring now.
