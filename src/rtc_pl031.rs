// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2022 Akira Moroo

use atomic_refcell::AtomicRefCell;
use chrono::{DateTime, Datelike, Timelike};
use r_efi::efi::MemoryDescriptor;

use crate::{
    arch::aarch64::layout::map,
    efi::ALLOCATOR,
    mem::{self, MemoryRegion},
};

static RTC: AtomicRefCell<Pl031> = AtomicRefCell::new(Pl031::new(map::mmio::PL031_START));

struct Pl031 {
    region: mem::MemoryRegion,
}

impl Pl031 {
    const RTCDR: u64 = 0x000;

    pub const fn new(base: usize) -> Self {
        Self {
            region: mem::MemoryRegion::new(base as u64, 0x1000),
        }
    }

    fn read_timestamp(&self) -> u32 {
        self.region.io_read_u32(Self::RTCDR)
    }

    pub fn read_date(&self) -> Result<(u8, u8, u8), ()> {
        let timestamp = self.read_timestamp();
        let datetime = DateTime::from_timestamp(timestamp as i64, 0).ok_or(())?;
        let date = datetime.date_naive();
        Ok((
            (date.year() - 2000) as u8,
            date.month() as u8,
            date.day() as u8,
        ))
    }

    pub fn read_time(&self) -> Result<(u8, u8, u8), ()> {
        let timestamp = self.read_timestamp();
        let datetime = DateTime::from_timestamp(timestamp as i64, 0).ok_or(())?;
        let time = datetime.time();
        Ok((time.hour() as u8, time.minute() as u8, time.second() as u8))
    }
}

pub fn read_date() -> Result<(u8, u8, u8), ()> {
    RTC.borrow_mut().read_date()
}

pub fn read_time() -> Result<(u8, u8, u8), ()> {
    RTC.borrow_mut().read_time()
}

/// When the OS remaps virtual-pages, we need to adjust our MMIO region.
pub fn fix_up(descriptors: &[MemoryDescriptor]) {
    let l = RTC.borrow_mut().region.as_bytes().len() as u64;
    let new_base = ALLOCATOR
        .borrow()
        .convert_internal_pointer(
            descriptors,
            RTC.borrow_mut().region.as_bytes().as_ptr() as u64,
        )
        .unwrap();
    let new_region = MemoryRegion::new(new_base, l);
    RTC.borrow_mut().region = new_region;
}
