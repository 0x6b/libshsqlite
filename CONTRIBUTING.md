# Contributing

## Project Structure

- [`soracom_harvest_api_client`](soracom_harvest_api_client): Simple API client for Soracom Harvest Data.
- [`soracom_harvest_client`](soracom_harvest_client): Simple command-line client for Soracom Harvest Data, can be used for testing as binary and library.
- [`soracom_harvest_sqlite_extension`](soracom_harvest_sqlite_extension): A SQLite extension for Soracom Harvest Data.

## Hacking

### Setup Development Prerequisites

- [SQLite](https://www.sqlite.org) 3.40.0
- [Rust](https://www.rust-lang.org) 1.65.0 (stable-aarch64-apple-darwin)
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen) 0.60.1

### Fork on GitHub

Before you do anything else, login on [GitHub](https://github.com/) and [fork](https://help.github.com/articles/fork-a-repo/) this repository.

### Clone Your Fork Locally

Install [Git](https://git-scm.com/) and clone your forked repository locally.

```shell
$ git clone https://github.com/<your-account>/libshsqlite
```

### Play with Your Fork

Create a new feature branch and play with it.

```shell
$ git switch -c add-new-feature
```

The project uses [Semantic Versioning 2.0.0](http://semver.org/), but you don't have to update `Cargo.toml` as I will maintain release.

#### Note: How to Update SQLite Binding

```shell
$ bindgen --default-macro-constant-type signed sqlite3ext.h -o sqlite3ext.rs
```

### Test Your Fork

In order to run the tests, you have to have working (virtual) SIM with Harvest Data enabled.

```shell
$ export LIBSHSQLITE_AUTH_KEY_ID=keyId-..
$ export LIBSHSQLITE_AUTH_KEY_SECRET=secret-...
$ export LIBSHSQLITE_TEST_IMSI=44010...
$ export LIBSHSQLITE_TEST_ENDPOINT=japan # or "global"
$ cargo build # the e2e test loads an extension from target/debug directory
$ cargo test
```

### Open a Pull Request

1. Commit your changes locally, [rebase onto upstream/main](https://github.com/blog/2243-rebase-and-merge-pull-requests), then push the changes to GitHub
   ```shell
   $ git push origin add-new-feature
   ```
2. Go to your fork on GitHub, switch to your feature branch, then click "Compare and pull request" button for review.

## References

- [Run-Time Loadable Extensions](https://www.sqlite.org/loadext.html)
- [The Virtual Table Mechanism Of SQLite](https://sqlite.org/vtab.html)
