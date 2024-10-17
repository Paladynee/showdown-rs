# showdown-rs

a simple multiplayer 2D shooter game server written entirely in Rust.
the game itself is also included, lol (written in JavaScript)

# Usage

clone the repository,

```
git clone https://github.com/Paladynee/showdown-rs.git
```

cd into the new directory

```
cd showdown-rs
```

to run the program,

```
cargo run --release
``` 

by default it will ask which network interface you want to host the server on.

running with.

```
cargo run --release prod
```

will host the server in whatever the first network interface `local_ip_address::local_ip()` returns;

running with,
```
cargo run --release dev
```

will host the server in `127.0.0.1`

simply connect to `http://ip_address:8080` to start playing (with your friends, if you have any).

