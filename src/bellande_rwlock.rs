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

pub mod architecture;
pub mod error;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::architecture::architecture::BellandeRwLock;

fn main() {
    let rwlock = Arc::new(BellandeRwLock::new(0));

    let rwlock_clone = Arc::clone(&rwlock);
    let handle = thread::spawn(move || {
        let mut write_guard = rwlock_clone.write();
        *write_guard += 1;
        thread::sleep(Duration::from_secs(1));
    });

    let rwlock_clone = Arc::clone(&rwlock);
    let handle2 = thread::spawn(move || {
        let read_guard = rwlock_clone.read();
        println!("Read value: {}", *read_guard);
    });

    handle.join().unwrap();
    handle2.join().unwrap();
}
