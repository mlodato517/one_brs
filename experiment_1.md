# First Experiment: Copying Files

I wanted to investigate the performance of various options for copying files.
This turned into a bit of a rabbit hole.

## Mechanisms

The actual, underlying mechanism varies by approach, and destination:

| Approach                  | Destination    | Underlying Mechanism                        |
|---------------------------|----------------|---------------------------------------------|
| `cat`                     | /dev/null      | reads/"writes" 256KiB                       |
| `cp`                      | /dev/null      | reads/"writes" 256KiB                       |
| `std::io::copy`           | /dev/null      | `sendfile` 2_147_479_552 byte chunks        |
| `std::io::Read`/`Write`   | /dev/null      | reads/"writes" whatever you pick            |
| `Mmap` + `std::io::copy`  | /dev/null      | "writes" 2_147_479_552 byte chunks          |
| `Mmap` + `std::io::Write` | /dev/null      | "writes" whatever you pick                  |
| `cat`                     | regular file   | `copy_file_range` 2_147_479_552 byte chunks |
| `cp`                      | regular file   | `ioctl(FICLONE)`                            |
| `std::io::copy`           | regular file   | `copy_file_range` 1_073_741_824 byte chunks |
| `std::io::Read`/`Write`   | regular file   | reads/writes whatever you pick              |
| `Mmap` + `std::io::copy`  | regular file   | writes 2_147_479_552 byte chunks            |
| `Mmap` + `std::io::Write` | regular file   | writes whatever you pick                    |

### Notes

1. The 256KiB value for `cat`/`cp` is often misquoted as 128KiB because it
   [changed relatively recently][coreutils_blksize].
2. The 2_147_479_552 byte size comes from [linux's max Read/Write
   value][max_linux_rw].
3. `ioctl(FICLONE)` is for [copy-on-write symlinking][ficlone], so no real data
   moves happen.
4. The `1GB` file cap in Rust's `copy_file_range()` is to [avoid `EOVERFLOW` in
   certain cases][rust_eoverflow].

### Open Questions

1. Why does Rust `sendfile()` to `/dev/null` when `cat`/`cp` don't? Is that
   "illegal"?
2. While there is a commit changing it from 2GB to 1GB, should Rust be copying
   files in 2_147_479_552 byte chunks?

## Un-cached Timings

Using our ~16GB file:

| Approach                  | Destination  | Mean [ms]       | Notes                                |
|---------------------------|--------------|-----------------| -------------------------------------|
| `Mmap` + `std::io::Write` | /dev/null    | 11.5 ± 1.5      | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | /dev/null    | 11.6 ± 1.4      | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | /dev/null    | 12.4 ± 3.3      |                                      |
| `Mmap` + `std::io::Write` | /dev/null    | 21.2 ± 3.2      | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 22.4 ± 3.5      | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 27.9 ± 3.7      | `Advise::Sequential` + 128KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 29.1 ± 3.1      | No `Advise` + 128KiB writes          |
| `cat`                     | regular file | 304.7 ± 24.2    |                                      |
| `std::io::copy`           | regular file | 323.6 ± 12.3    |                                      |
| `cp`                      | regular file | 380.0 ± 123.1   |                                      |
| `std::io::copy`           | /dev/null    | 6500.4 ± 42.2   |                                      |
| `std::io::Read`/`Write`   | /dev/null    | 7065.0 ± 53.2   | 256KiB reads/writes                  |
| `std::io::Read`/`Write`   | /dev/null    | 7076.1 ± 102.3  | 128KiB reads/writes                  |
| `cp`                      | /dev/null    | 7191.6 ± 32.5   |                                      |
| `cat`                     | /dev/null    | 7229.7 ± 64.4   |                                      |
| `Mmap` + `std::io::Write` | regular file | 14948.0 ± 124.8 | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | regular file | 14958.7 ± 94.8  |                                      |
| `Mmap` + `std::io::Write` | regular file | 14965.0 ± 186.3 | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | regular file | 15091.2 ± 92.2  | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 15097.2 ± 72.8  | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | regular file | 15270.1 ± 58.2  | `Advise::Sequential` + 128KiB writes |
| `Mmap` + `std::io::Write` | regular file | 15291.1 ± 86.7  | No `Advise` + 128KiB writes          |
| `std::io::Read`/`Write`   | regular file | 19049.2 ± 122.8 | 256KiB reads/writes                  |
| `std::io::Read`/`Write`   | regular file | 19547.3 ± 208.6 | 128KiB reads/writes                  |

## Cached Timings

Using our ~16GB file:

| Approach                  | Destination  | Mean [ms]      | Notes                                |
|---------------------------|--------------|----------------|--------------------------------------|
| `Mmap` + `std::io::Write` | /dev/null    | 0.5 ± 0.1      | No `Advise` + max writes             |
| `Mmap` + `std::io::Write` | /dev/null    | 0.5 ± 0.1      | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::copy`  | /dev/null    | 0.5 ± 0.1      |                                      |
| `Mmap` + `std::io::Write` | /dev/null    | 7.6 ± 1.5      | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 7.7 ± 1.6      | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 13.8 ± 1.5     | No `Advise` + 128KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 14.7 ± 2.0     | `Advise::Sequential` + 128KiB writes |
| `std::io::copy`           | /dev/null    | 152.7 ± 5.6    |                                      |
| `cp`                      | regular file | 195.0 ± 4.8    |                                      |
| `std::io::copy`           | regular file | 195.3 ± 4.2    |                                      |
| `cat`                     | regular file | 195.9 ± 4.0    |                                      |
| `cat`                     | /dev/null    | 1152.7 ± 15.6  |                                      |
| `cp`                      | /dev/null    | 1159.1 ± 11.0  |                                      |
| `std::io::Read`/`Write`   | /dev/null    | 1162.0 ± 16.1  | 128KiB writes                        |
| `std::io::Read`/`Write`   | /dev/null    | 1182.3 ± 17.3  | 256KiB writes                        |
| `Mmap` + `std::io::Write` | regular file | 9213.7 ± 93.8  | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | regular file | 9223.6 ± 53.0  | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | regular file | 9253.5 ± 81.4  |                                      |
| `std::io::Read`/`Write`   | regular file | 9362.3 ± 398.6 | 128KiB reads/writes                  |
| `std::io::Read`/`Write`   | regular file | 9478.2 ± 161.2 | 256KiB reads/writes                  |
| `Mmap` + `std::io::Write` | regular file | 9592.6 ± 96.9  | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 9782.6 ± 583.1 | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | regular file | 9802.2 ± 70.7  | No `Advise` + 128KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 9817.7 ± 116.8 | `Advise::Sequential` + 128KiB writes |

Note that most things experience a huge slowdown when they have to actually do
filesystem writes to regular files except for `cat` and `cp` which use other
mechanisms!

### Helpful Note on strace

I used `strace` to figure out what most things were doing:

```sh
strace cat measurements.txt > measurements2.txt > strace_cat_file_to_file.txt
```

But the redirects were all messed up. I found [an SO answer][strace]
recommending:

```sh
strace -o strace_cat_file_to_file.txt cat measurements.txt > measurements2.txt
```

[coreutils_blksize]: https://github.com/coreutils/coreutils/commit/fcfba90d0d27a1bacf2020bac4dbec74ed181028
[ficlone]: https://man7.org/linux/man-pages/man2/ioctl_ficlonerange.2.html
[max_linux_rw]: https://stackoverflow.com/a/70370002
[rust_eoverflow]: https://github.com/rust-lang/rust/commit/bbfa92c82debed28417350b15fc6a2f46135346d
[strace]: https://stackoverflow.com/questions/70556877/how-to-conver-strace-o-cat-outputfile-command-args-to-csh
