use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        while let Err(_) = self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
            while self.locked.load(Ordering::Relaxed) {
                // Spin until the lock is released
                unsafe { asm!("pause") };
            }
        }
    
        SpinlockGuard {
            spinlock: &self,
        }
    }
    
}

pub struct SpinlockGuard<'a, T> {
    spinlock: &'a Spinlock<T>,
}

impl<'a, T> core::ops::Deref for SpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.spinlock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.spinlock.data.get() }
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.spinlock.locked.store(false, Ordering::Release);
    }
}
