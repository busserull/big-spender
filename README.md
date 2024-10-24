# Big Spender

Big Spender is a simple way to automate cost division
between any number of participants, in any number of
currencies, using a json file as input.

The bare minimum amount of effort has been put into this,
so it is not mathematically guaranteed to find the least
number of balancing transactions - but in real life
scenarios it will typically still do, or find a solution
that uses two to three extra transactions.

It has mostly been developed as a pretty ad-hoc solution
to cost division after diving trips for NTNUI DG, and
since the tool itself is dead simple, I won't even
bother to document the particular json keys and values
supported. Just look at the example `fd24.json` file,
and you will pick it up in no time.

## Running

TLDR:
```
cargo run -- fd24.json
```

To see all supported options:
```
cargo run -- -h
```
