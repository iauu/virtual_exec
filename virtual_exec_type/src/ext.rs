use alloc::sync::Arc;
use async_lock::{Mutex, MutexGuardArc, RwLock, RwLockReadGuardArc, RwLockWriteGuardArc};

pub trait SafeReadArcExt<T> {
    fn read_arc_safe(&self) -> RwLockReadGuardArc<T>;
}

pub trait SafeLockArcExt<T> {
    fn lock_arc_safe(&self) -> MutexGuardArc<T>;
}

pub trait SafeWriteArcExt<T> {
    fn write_arc_safe(&self) -> RwLockWriteGuardArc<T>;
}


impl<T> SafeReadArcExt<T> for Arc<RwLock<T>> {
    #[cfg(feature = "std")]
    #[inline]
    fn read_arc_safe(&self) -> RwLockReadGuardArc<T> {
        self.read_arc_blocking()
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    fn read_arc_safe(&self) -> RwLockReadGuardArc<T> {
        self.try_read_arc().expect("Deadlock")
    }
}

impl<T> SafeLockArcExt<T> for Arc<Mutex<T>> {
    #[cfg(feature = "std")]
    #[inline]
    fn lock_arc_safe(&self) -> MutexGuardArc<T> {
        self.lock_arc_blocking()
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    fn lock_arc_safe(&self) -> MutexGuardArc<T> {
        self.try_lock_arc().expect("Deadlock")
    }
}

impl<T> SafeWriteArcExt<T> for Arc<RwLock<T>> {
    #[cfg(feature = "std")]
    #[inline]
    fn write_arc_safe(&self) -> RwLockWriteGuardArc<T> {
        self.write_arc_blocking()
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    fn write_arc_safe(&self) -> RwLockWriteGuardArc<T> {
        self.try_write_arc().expect("Deadlock")
    }
}