# libshsqlite

A SQLite extension which loads data from Soracom Harvest Data as a virtual table.

## Tested Platform

- [SQLite](https://www.sqlite.org) 3.40.0
- [Rust](https://www.rust-lang.org) 1.65.0 (stable-aarch64-apple-darwin)
- macOS 12.6 (Monterey) on Apple M1 MAX
- SORACOM Harvest ([English](https://www.soracom.io/products/harvest/)/[Japanese](https://soracom.jp/services/harvest/))

## Getting Started

### Clone the Repository

```shell
$ git clone github.com/0x6b/libshsqlite
$ cd libshsqlite
```

### Setup and Soracom Harvest Data

1. Sign-up SORACOM to get a (virtual) SIM ([English](https://www.soracom.io/)/[Japanese](https://www.soracom.jp/))
2. Enable Soracom Harvest Data ([English](https://developers.soracom.io/en/docs/harvest/configuration/#harvest-data)/[Japanese](https://users.soracom.io/ja-jp/docs/harvest/enable-data/))
3. Create a SAM user with following permission:
   ```json
   {
     "statements": [
       {
         "api": [
           "Sim:getDataFromSim",
           "Subscriber:getDataFromSubscriber",
           "DataEntry:getDataEntries",
           "DataEntry:getDataEntry"
         ],
         "effect": "allow"
       }
     ]
   }
   ```
4. Generate and authentication information for the user and save it for future reference

### Send Some Data

See documentation ([English](https://developers.soracom.io/en/docs/harvest/collecting-data/)/[Japanese](https://users.soracom.io/ja-jp/docs/harvest/send-data/)) for detail, or use deadly simple client in this repository ([`soracom_harvest_client`](soracom_harvest_client)), from your SIM or virtual SIM connected machine as follows:

```shell
$ cargo run -p soracom_harvest_client -- --udp hey # Say hello, via UDP
```

### Build the Extension

```shell
$ cargo build --release
```

### Load the Extension

1. Export required environment variables with the credential:
   ```shell
   $ export LIBSHSQLITE_AUTH_KEY_ID=keyId-.. # authKeyId
   $ export LIBSHSQLITE_AUTH_KEY_SECRET=secret-... # authKey
   ```
2. Launch SQLite:
   ```shell
   $ sqlite3
   ```
3. Load the extension (you have to use "shsqlite" on Windows):
   ```shell
   .load target/release/libshsqlite
   ```
   If you get `Error: unknown command or invalid arguments: "load". Enter ".help" for help `, your SQLite is not capable for loading an extension. For macOS, install it with `brew install sqlite3`, and use it.

### Query Your Data

1. Create a virtual table for your Soracom Harvest Data by providing `IMSI` (IMSI of target SIM) as module argument. See reference below for available arguments other than `IMSI`:
   ```sql
   CREATE VIRTUAL TABLE harvest_data USING shsqlite(IMSI 'imsi-of-your-sim', COVERAGE 'japan');
   ```
2. Run queries as usual:
   ```sql
   SELECT * FROM harvest_data;
   SELECT * FROM harvest_data WHERE value ->>'$.temperature' > 10;
   ```

## Module Arguments Reference

| Argument   | Description                                                               | Default             | Required |
|------------|---------------------------------------------------------------------------|---------------------|:--------:|
| `IMSI`     | Your IMSI                                                                 | None                |    x     |
| `FROM`     | Start time for the data entries search range (unix time in milliseconds). | 1 days ago from now |          |
| `TO`       | End time for the data entries search range (unix time in milliseconds).   | now                 |          |
| `COVERAGE` | Your SIM's coverage (`global` or `japan`)                                 | `global`            |          |
| `LIMIT`    | Maximum number of data entries to retrieve. Should be between 1 and 1000. | 100                 |          |

```sql
CREATE VIRTUAL TABLE harvest_data USING shsqlite(
    IMSI '...',
    FROM '...',
    TO '...',
    COVERAGE 'japan|global',
    LIMIT '...'
);
```

## Contributing

Please read [CONTRIBUTING](CONTRIBUTING.md) for more detail.

# Acknowledgements

An article, [Extending SQLite with Rust to support Excel files as virtual tables | Sergey Khabibullin](https://sergey.khabibullin.com/sqlite-extensions-in-rust/) , and its companion repository [x2bool/xlite](https://github.com/x2bool/xlite), for great write up and inspiration.

# Limitations

- The extension will load the data only once while creating a virtual table. If you want to pick up recent data, drop the table and create it again. Dropping the table won't erase your data on Soracom Harvest.
- `INSERT`, `UPDATE` and `DELETE` statements won't be implemented.

# Privacy

The extension never send your data to any server.

# License

This extension is released under the MIT License. See [LICENSE](LICENSE) for details.
