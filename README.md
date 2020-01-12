# Troll
Troll is an analyzer of the DWARF information embeded in ELF binary.
This is inspired by [fromelf](http://www.keil.com/support/man/docs/armutil/default.htm), and written in [Rust](https://www.rust-lang.org/).

## Features
- Output static variable information

## Install
- Downloads binaries from [Release](https://github.com/forestaa/troll/releases)

## Examples
```
$ git clone https://github.com/forestaa/troll.git
$ cat examples/simple.c
#include <stdio.h>

typedef struct hoge {
  int hoge;
  char hogehoge;
  int array[2];
} Hoge;

Hoge hoges[3];

int main(void) {
  return 0;
}
$ gcc -O0 -g -o examples/simple examples/simple.c
$ cargo run examples/simple  # You can run `./troll examples/simple` if you install a binary.
...

address    size  variable_name        type
0x00004060 0x012 hoges                Hoge[2]
0x00004060 0x009 hoges[0]             Hoge
0x00004060 0x004 hoges[0].hoge        int
0x00004064 0x001 hoges[0].hogehoge    char
0x00004065 0x004 hoges[0].array       int[1]
0x00004065 0x004 hoges[0].array[0]    int
0x00004069 0x009 hoges[1]             Hoge
0x00004069 0x004 hoges[1].hoge        int
0x0000406d 0x001 hoges[1].hogehoge    char
0x0000406e 0x004 hoges[1].array       int[1]
0x0000406e 0x004 hoges[1].array[0]    int

```
