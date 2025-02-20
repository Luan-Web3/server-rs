Server + Hyper
===========================

![GitHub repo size](https://img.shields.io/github/repo-size/Luan-Web3/server-rs?style=for-the-badge)
![GitHub language count](https://img.shields.io/github/languages/count/Luan-Web3/server-rs?style=for-the-badge)
![GitHub forks](https://img.shields.io/github/forks/Luan-Web3/server-rs?style=for-the-badge)

Simple rust project in which you can create a server using [hyper.rs](https://hyper.rs/).

## Prerequisites

Before you begin, make sure you meet the following requirements:

- [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)

## Instructions

```
git clone https://github.com/Luan-Web3/server-rs.git
```
```
cd server-rs
```
```
cargo build
```
```
cargo run
```

## Examples

### GET /people
```
curl -X GET http://localhost:3000/people
```

### POST /people
```
curl -X POST http://localhost:3000/people -H "Content-Type: application/json" -d '{"name": "John"}'
```

### PUT /people

```
curl -X PUT http://localhost:3000/people/:id -H "Content-Type: application/json" -d '{"name": "John Doe"}'
```

### DELETE /people

```
curl -X DELETE http://localhost:3000/people/:id
```
## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
