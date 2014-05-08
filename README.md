Rust-Langton-s-Ant
==================

Try Rust with a Langton's Ant implementation.


Usage
-----

~~~bash
$ ./main -h
Usage: ./main [options]

Langton's Ant rust implementation.

Options:
    -m --map-size WIDTHxHEIGHT
                        the map size
    -w --window-size WIDTHxHEIGHT
                        the window resolution
    -r --refresh TIME   interval between each refresh (ms)
    -c --cycle TIME     interval between each cycle (ms)
    -h --help           display this help

~~~

Compilation
-----

libsdl-dev is required.

~~~bash
$ git submodule init
$ git submodule update
$ make

~~~
