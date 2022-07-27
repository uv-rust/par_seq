// Need to move pointer to buffer element across threads
//-----------------------------------------------------------------------------
struct Movable<T>(*const T);
impl<T> Movable<T> {
    fn get(&self) -> Option<*const T> {
        if self.0.is_null() {
            return None;
        }
        Some(self.0)
    }
}

struct MovableMut<T>(*mut T);
impl<T> MovableMut<T> {
    fn get(&self) -> Option<*mut T> {
        if self.0.is_null() {
            return None;
        }
        Some(self.0)
    }
}

unsafe impl<T> Send for Movable<T> {}
unsafe impl<T> Send for MovableMut<T> {}

//-----------------------------------------------------------------------------
// Structs to move callable objects across threads, 1 and 2 arg versions
//-----------------------------------------------------------------------------
type KernelFun2<T> = dyn Fn(&[T], &mut [T]);
struct FnMove2<T> {
    f: std::sync::Arc<KernelFun2<T>>,
}
impl<T> FnMove2<T> {
    fn call(&self, src: &[T], dest: &mut [T]) {
        (self.f)(src, dest);
    }
}
unsafe impl<T> Send for FnMove2<T> {}
//-----------------------------------------------------------------------------
type KernelFun1<T> = dyn Fn(&mut [T]);
struct FnMove1<T> {
    f: std::sync::Arc<KernelFun1<T>>,
}
impl<T> FnMove1<T> {
    fn call(&self, dest: &mut [T]) {
        (self.f)(dest);
    }
}
unsafe impl<T> Send for FnMove1<T> {}

//-----------------------------------------------------------------------------
#[macro_export]
macro_rules! kernel {
    ( $x:expr ) => {{
        std::sync::Arc::new($x)
    }};
}

//-----------------------------------------------------------------------------
pub fn par_map<T: 'static>(
    src: &[T],
    dest: &mut [T],
    num_threads: usize,
    fr: std::sync::Arc<KernelFun2<T>>,
) -> std::thread::Result<()> {
    let mut th = vec![];
    let chunk_size = (src.len() + num_threads - 1) / num_threads;
    let last_chunk_size = src.len() - (chunk_size * (num_threads - 1));
    for i in 0..num_threads {
        unsafe {
            let idx = (chunk_size * i) as isize;
            let cs = if i < num_threads - 1 {
                chunk_size
            } else {
                last_chunk_size
            };
            let s = Movable(src.as_ptr().offset(idx));
            let d = MovableMut(dest.as_mut_ptr().offset(idx));
            let k = FnMove2 { f: fr.clone() };
            th.push(std::thread::spawn(move || {
                let src = std::slice::from_raw_parts(s.get().unwrap(), cs);
                let mut dst = std::slice::from_raw_parts_mut(d.get().unwrap(), cs);
                k.call(&src, &mut dst);
            }));
        }
    }
    for t in th {
        if let Err(e) = t.join() {
            return Err(e);
        }
    }
    Ok(())
}

//-----------------------------------------------------------------------------
pub fn par_in_place_map<T: 'static>(
    dest: &mut [T],
    num_threads: usize,
    fr: std::sync::Arc<KernelFun1<T>>,
) -> std::thread::Result<()> {
    let mut th = vec![];
    let chunk_size = (dest.len() + num_threads - 1) / num_threads;
    let last_chunk_size = dest.len() - (chunk_size * (num_threads - 1));
    for i in 0..num_threads {
        unsafe {
            let idx = (chunk_size * i) as isize;
            let cs = if i < num_threads - 1 {
                chunk_size
            } else {
                last_chunk_size
            };
            let d = MovableMut(dest.as_mut_ptr().offset(idx));
            let k = FnMove1 { f: fr.clone() };
            th.push(std::thread::spawn(move || {
                let mut dst = std::slice::from_raw_parts_mut(d.get().unwrap(), cs);
                k.call(&mut dst);
            }));
        }
    }
    for t in th {
        if  let Err(e) = t.join() {
            return Err(e);
        }
    }
    Ok(())
}

//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
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
    #[test]
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
}
