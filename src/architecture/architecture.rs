// Copyright (C) 2025 Bellande Architecture Mechanism Research Innovation Center, Ronaldson Bellande

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

pub struct BellandeRwLock<T> {
    data: Mutex<RefCell<T>>,
    readers: AtomicUsize,
    writer: Mutex<()>,
    condvar: Condvar,
}

impl<T> BellandeRwLock<T> {
    pub fn new(data: T) -> Self {
        BellandeRwLock {
            data: Mutex::new(RefCell::new(data)),
            readers: AtomicUsize::new(0),
            writer: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let readers = self.readers.load(Ordering::Acquire);

        if readers == 0 {
            let _writer_guard = self.writer.lock().unwrap();
            let current_readers = self.readers.load(Ordering::Acquire);
            if current_readers == 0 {
                self.readers.store(1, Ordering::Release);
            } else {
                self.readers.fetch_add(1, Ordering::Acquire);
            }
        } else {
            self.readers.fetch_add(1, Ordering::Acquire);
        }

        ReadGuard::new(self)
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        let mut writer_guard = self.writer.lock().unwrap();
        while self.readers.load(Ordering::Acquire) > 0 {
            writer_guard = self.condvar.wait(writer_guard).unwrap();
        }
        WriteGuard::new(self, writer_guard)
    }
}

pub struct ReadGuard<'a, T> {
    rwlock: &'a BellandeRwLock<T>,
    _mutex_guard: std::sync::MutexGuard<'a, RefCell<T>>,
}

impl<'a, T> ReadGuard<'a, T> {
    fn new(rwlock: &'a BellandeRwLock<T>) -> Self {
        let mutex_guard = rwlock.data.lock().unwrap();
        ReadGuard {
            rwlock,
            _mutex_guard: mutex_guard,
        }
    }
}

impl<'a, T> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        let readers = self.rwlock.readers.fetch_sub(1, Ordering::Release);
        if readers == 1 {
            self.rwlock.condvar.notify_all();
        }
    }
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self._mutex_guard.as_ptr() }
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a BellandeRwLock<T>,
    _mutex_guard: std::sync::MutexGuard<'a, RefCell<T>>,
    _writer_guard: std::sync::MutexGuard<'a, ()>,
}

impl<'a, T> WriteGuard<'a, T> {
    fn new(rwlock: &'a BellandeRwLock<T>, writer_guard: std::sync::MutexGuard<'a, ()>) -> Self {
        let mutex_guard = rwlock.data.lock().unwrap();
        WriteGuard {
            rwlock,
            _mutex_guard: mutex_guard,
            _writer_guard: writer_guard,
        }
    }
}

impl<'a, T> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        self.rwlock.condvar.notify_all();
    }
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self._mutex_guard.as_ptr() }
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self._mutex_guard.as_ptr() }
    }
}
