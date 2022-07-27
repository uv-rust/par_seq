# Parallel Sequences

Simple parallel processing of sequences by providing an `Fn` object executed on sub-ranges.
Copy and in-place modification supported.

## Map from copy
```rust
    fn par_map_test() -> std::thread::Result<()> {
        let len = 64;
        let src = vec![0_u8; len];
        let mut dest = vec![0_u8; len];
        let x = 1;
        let kernel_fun = move |s: &[u8], d: &mut [u8]| {
            for i in 0..s.len() {
                d[i] = s[i] + x;
            }
        };
        if let Err(e) = par_map(&src, &mut dest, 3, kernel!(kernel_fun)) {
            return Err(e);
        }
        for e in dest {
            assert_eq!(e, 1);
        }
        Ok(())
    }
```


## In-place modification
```rust
    fn par_in_place_map_test() -> std::thread::Result<()> {
        let len = 64;
        let mut dest = vec![0_u8; len];
        let x = 1;
        let kernel_fun = move |d: &mut [u8]| {
            for i in 0..d.len() {
                d[i] += x;
            }
        };
        if let Err(e) = par_in_place_map(&mut dest, 3, kernel!(kernel_fun)) {
            return Err(e);
        }
        for e in dest {
            assert_eq!(e, 1);
        }
        Ok(())
    }
```
