use pin_init::*;
use pin_project::pin_project;

use std::mem::MaybeUninit;
use std::pin::Pin;
use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};
use std::{io::Error, marker::PhantomPinned};

#[repr(transparent)]
struct RawMutex {
    pthread: UnsafeCell<libc::pthread_mutex_t>,
}

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

struct RawMutexGuard<'a>(&'a RawMutex);

impl Drop for RawMutex {
    fn drop(&mut self) {
        // Thanks to pin_init, we are certain that self is initialized already.
        unsafe {
            println!("drop");
            libc::pthread_mutex_destroy(self.pthread.get());
        }
    }
}

impl RawMutex {
    // Use pin_init's abstraction to provide a safe initializaiton.
    pub fn new() -> impl Init<Self, Error> {
        init_from_closure(|mut this| {
            let ptr = this.get_mut().as_mut_ptr() as *mut libc::pthread_mutex_t;
            unsafe {
                ptr.write(libc::PTHREAD_MUTEX_INITIALIZER);

                let mut attr = MaybeUninit::<libc::pthread_mutexattr_t>::uninit();
                let ret = libc::pthread_mutexattr_init(attr.as_mut_ptr());
                if ret != 0 {
                    return Err(this.init_err(Error::from_raw_os_error(ret)));
                }

                let ret =
                    libc::pthread_mutexattr_settype(attr.as_mut_ptr(), libc::PTHREAD_MUTEX_NORMAL);
                if ret != 0 {
                    libc::pthread_mutexattr_destroy(attr.as_mut_ptr());
                    return Err(this.init_err(Error::from_raw_os_error(ret)));
                }

                let ret = libc::pthread_mutex_init(ptr, attr.as_ptr());
                libc::pthread_mutexattr_destroy(attr.as_mut_ptr());
                if ret != 0 {
                    return Err(this.init_err(Error::from_raw_os_error(ret)));
                }

                Ok(this.init_ok())
            }
        })
    }

    pub fn lock(&self) -> RawMutexGuard<'_> {
        unsafe {
            libc::pthread_mutex_lock(self.pthread.get());
            RawMutexGuard(self)
        }
    }
}

impl Drop for RawMutexGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            libc::pthread_mutex_unlock(self.0.pthread.get());
        }
    }
}

// Now we can use pin_init to define a Mutex that wraps the RawMutex
#[pin_init]
struct Mutex<T> {
    #[pin]
    mutex: RawMutex,
    #[pin]
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

struct MutexGuard<'a, T>(RawMutexGuard<'a>, &'a mut T);

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.1
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.1
    }
}

impl<T> Mutex<T> {
    pub fn new<F>(value: F) -> impl Init<Self, Error>
    where
        F: Init<T, Error>,
    {
        init_pin!(Mutex {
            mutex <- RawMutex::new(),
            data: init_pin!(UnsafeCell {
                0 <- value
            }),
        })
    }

    pub fn lock(&self) -> Pin<MutexGuard<'_, T>> {
        let g = self.mutex.lock();
        unsafe { Pin::new_unchecked(MutexGuard(g, &mut *self.data.get())) }
    }

    pub fn get_inner(self: Pin<&mut Self>) -> Pin<&mut T> {
        unsafe { Pin::new_unchecked(&mut *self.data.get()) }
    }
}

// Can be used on tuple structs.
// Can be used together with pin_project.
#[pin_init]
#[pin_project]
struct Pinned<T>(T, #[pin] PhantomPinned);

#[pin_init]
#[pin_project]
struct TwoMutex {
    #[pin]
    a: Mutex<i32>,
    #[pin]
    b: Mutex<Pinned<i32>>,
}

fn main() {
    {
        let m = Box::pin_with(Mutex::new(1)).unwrap();
        println!("{}", *m.lock());
        *m.lock() = 2;
        println!("{}", *m.lock());
        *m.lock() = 3;
        println!("{}", *m.lock());
    }

    {
        // Even complex, nested pinned data structure can be safely
        // created and initialized on the stack.
        init_stack!(
            m = TwoMutex {
                a <- Mutex::new(1),
                b <- Mutex::new(Pinned(1, PhantomPinned))
            }
        );
        let mut m = m.unwrap();

        println!("{}", *m.a.lock());
        *m.a.lock() = 2;
        // Use pin-projection to access unpinned fields
        *m.b.lock().as_mut().project().0 = 2;
        println!("{}", m.b.lock().0);
        // Use pin-projection to access pinned fields.
        *m.as_mut().project().b.get_inner().project().0 = 3;
        println!("{}", m.b.lock().0);
    }
}
