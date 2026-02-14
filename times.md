# Timings for our 1brc

## File Stats

```sh
➜ stat measurements.txt
  File: measurements.txt
  Size: 15931636665     Blocks: 31116480   IO Block: 4096   regular file
Device: 0,45    Inode: 5097011     Links: 1
Access: (0644/-rw-r--r--)  Uid: ( 1000/ mlodato)   Gid: ( 1000/ mlodato)
Context: unconfined_u:object_r:user_home_t:s0
Access: 2026-02-13 20:10:46.489513449 -0500
Modify: 2026-02-13 20:04:56.583264293 -0500
Change: 2026-02-13 20:09:34.037357316 -0500
 Birth: 2026-02-13 19:56:52.574221635 -0500
```
## Timings

| Name                       | Time     | Deviation |
|----------------------------|----------|-----------|
| cat > /dev/null            | 1.489 s  | 0.039 s   |
| getting path arg           | 595.5 µs | 209.7 µs  |
| file len                   | 617.0 µs | 242.3 µs  |
| count line lens            | 1.518 s  | 0.044 s   |
| BufRead line lens          | 43.734 s | 0.495 s   |
| count line lens no Discord | 1.296    | 0.036     |
