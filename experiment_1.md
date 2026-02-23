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

| Approach                  | Destination  | Mean [ms]       | Min [ms] | Max [ms] | Relative         | Notes                                |
|---------------------------|--------------|-----------------|----------|----------|------------------| -------------------------------------|
| `Mmap` + `std::io::Write` | /dev/null    | 11.5 ± 1.5      | 9.5      | 15.3     | 1.00             | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | /dev/null    | 11.6 ± 1.4      | 9.9      | 14.7     | 1.01 ± 0.18      | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | /dev/null    | 12.4 ± 3.3      | 9.1      | 19.9     | 1.08 ± 0.32      |                                      |
| `Mmap` + `std::io::Write` | /dev/null    | 21.2 ± 3.2      | 15.3     | 28.1     | 1.85 ± 0.37      | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 22.4 ± 3.5      | 17.5     | 27.2     | 1.95 ± 0.40      | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 27.9 ± 3.7      | 24.2     | 37.3     | 2.44 ± 0.46      | `Advise::Sequential` + 128KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 29.1 ± 3.1      | 23.9     | 37.3     | 2.54 ± 0.43      | No `Advise` + 128KiB writes          |
| `cat`                     | regular file | 304.7 ± 24.2    | 271.2    | 334.3    | 26.60 ± 4.13     |                                      |
| `std::io::copy`           | regular file | 323.6 ± 12.3    | 309.3    | 346.2    | 28.24 ± 3.91     |                                      |
| `cp`                      | regular file | 380.0 ± 123.1   | 316.2    | 718.6    | 33.17 ± 11.62    |                                      |
| `std::io::copy`           | /dev/null    | 6500.4 ± 42.2   | 6431.4   | 6571.3   | 567.40 ± 75.74   |                                      |
| `std::io::Read`/`Write`   | /dev/null    | 7065.0 ± 53.2   | 6965.5   | 7130.1   | 616.68 ± 82.35   | 256KiB reads/writes                  |
| `std::io::Read`/`Write`   | /dev/null    | 7076.1 ± 102.3  | 6945.2   | 7224.5   | 617.65 ± 82.83   | 128KiB reads/writes                  |
| `cp`                      | /dev/null    | 7191.6 ± 32.5   | 7133.0   | 7241.1   | 627.73 ± 83.74   |                                      |
| `cat`                     | /dev/null    | 7229.7 ± 64.4   | 7088.2   | 7305.6   | 631.06 ± 84.32   |                                      |
| `Mmap` + `std::io::Write` | regular file | 14948.0 ± 124.8 | 14710.6  | 15096.8  | 1304.76 ± 174.29 | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | regular file | 14958.7 ± 94.8  | 14787.1  | 15084.6  | 1305.69 ± 174.27 |                                      |
| `Mmap` + `std::io::Write` | regular file | 14965.0 ± 186.3 | 14702.5  | 15295.0  | 1306.24 ± 174.91 | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | regular file | 15091.2 ± 92.2  | 14912.8  | 15203.5  | 1317.26 ± 175.80 | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 15097.2 ± 72.8  | 14986.5  | 15199.5  | 1317.79 ± 175.80 | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | regular file | 15270.1 ± 58.2  | 15194.3  | 15383.3  | 1332.88 ± 177.77 | `Advise::Sequential` + 128KiB writes |
| `Mmap` + `std::io::Write` | regular file | 15291.1 ± 86.7  | 15171.6  | 15393.8  | 1334.71 ± 178.11 | No `Advise` + 128KiB writes          |
| `std::io::Read`/`Write`   | regular file | 19049.2 ± 122.8 | 18846.3  | 19272.2  | 1662.74 ± 221.94 | 256KiB reads/writes                  |
| `std::io::Read`/`Write`   | regular file | 19547.3 ± 208.6 | 19117.6  | 19812.8  | 1706.22 ± 228.20 | 128KiB reads/writes                  |

## Cached Timings

Using our ~16GB file:

| Approach                  | Destination  | Mean [ms]      | Min [ms] | Max [ms] | Relative           | Notes                                |
|---------------------------|--------------|----------------|----------|----------|--------------------|--------------------------------------|
| `Mmap` + `std::io::Write` | /dev/null    | 0.5 ± 0.1      | 0.2      | 3.6      | 1.00               | No `Advise` + max writes             |
| `Mmap` + `std::io::Write` | /dev/null    | 0.5 ± 0.1      | 0.2      | 1.2      | 1.01 ± 0.41        | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::copy`  | /dev/null    | 0.5 ± 0.1      | 0.2      | 1.4      | 1.05 ± 0.42        |                                      |
| `Mmap` + `std::io::Write` | /dev/null    | 7.6 ± 1.5      | 5.8      | 14.0     | 15.53 ± 5.52       | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 7.7 ± 1.6      | 5.9      | 14.1     | 15.58 ± 5.67       | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | /dev/null    | 13.8 ± 1.5     | 11.0     | 19.4     | 28.17 ± 8.96       | No `Advise` + 128KiB writes          |
| `Mmap` + `std::io::Write` | /dev/null    | 14.7 ± 2.0     | 11.1     | 27.5     | 29.85 ± 9.83       | `Advise::Sequential` + 128KiB writes |
| `std::io::copy`           | /dev/null    | 152.7 ± 5.6    | 145.5    | 165.0    | 310.84 ± 93.74     |                                      |
| `cp`                      | regular file | 195.0 ± 4.8    | 187.1    | 207.0    | 396.96 ± 119.23    |                                      |
| `std::io::copy`           | regular file | 195.3 ± 4.2    | 187.1    | 202.6    | 397.56 ± 119.30    |                                      |
| `cat`                     | regular file | 195.9 ± 4.0    | 188.2    | 201.3    | 398.78 ± 119.65    |                                      |
| `cat`                     | /dev/null    | 1152.7 ± 15.6  | 1109.8   | 1163.3   | 2346.84 ± 703.20   |                                      |
| `cp`                      | /dev/null    | 1159.1 ± 11.0  | 1151.4   | 1189.2   | 2359.98 ± 706.77   |                                      |
| `std::io::Read`/`Write`   | /dev/null    | 1162.0 ± 16.1  | 1123.4   | 1172.5   | 2365.88 ± 708.94   | 128KiB writes                        |
| `std::io::Read`/`Write`   | /dev/null    | 1182.3 ± 17.3  | 1143.9   | 1214.4   | 2407.19 ± 721.42   | 256KiB writes                        |
| `Mmap` + `std::io::Write` | regular file | 9213.7 ± 93.8  | 9050.8   | 9344.9   | 18759.03 ± 5618.43 | `Advise::Sequential` + max writes    |
| `Mmap` + `std::io::Write` | regular file | 9223.6 ± 53.0  | 9140.3   | 9322.5   | 18779.13 ± 5622.23 | No `Advise` + max writes             |
| `Mmap` + `std::io::copy`  | regular file | 9253.5 ± 81.4  | 9162.4   | 9420.1   | 18839.95 ± 5641.84 |                                      |
| `std::io::Read`/`Write`   | regular file | 9362.3 ± 398.6 | 8254.0   | 9651.9   | 19061.63 ± 5763.18 | 128KiB reads/writes                  |
| `std::io::Read`/`Write`   | regular file | 9478.2 ± 161.2 | 9141.2   | 9680.3   | 19297.56 ± 5785.70 | 256KiB reads/writes                  |
| `Mmap` + `std::io::Write` | regular file | 9592.6 ± 96.9  | 9428.9   | 9753.9   | 19530.51 ± 5849.44 | No `Advise` + 256KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 9782.6 ± 583.1 | 9362.1   | 11395.4  | 19917.33 ± 6078.94 | `Advise::Sequential` + 256KiB writes |
| `Mmap` + `std::io::Write` | regular file | 9802.2 ± 70.7  | 9632.0   | 9868.4   | 19957.25 ± 5975.58 | No `Advise` + 128KiB writes          |
| `Mmap` + `std::io::Write` | regular file | 9817.7 ± 116.8 | 9643.9   | 10024.0  | 19988.81 ± 5988.02 | `Advise::Sequential` + 128KiB writes |

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
