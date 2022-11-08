// SPDX-License-Identifier: GPL-2.0

//! Power management regulator framework.
//!
//! C header: [`include/linux/regulator/consumer.h`](../../../../include/linux/regulator/consumer.h)
use crate::{bindings, device, error::Result, pr_err, str::CStr, to_result};

/// Represents `struct regulator *`.
///
/// # Invariants
///
/// The pointer is valid.
pub struct Regulator(*mut bindings::regulator);
impl Regulator {
    /// Creates new regulator structure from a raw pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be valid.
    pub unsafe fn new(regulator: *mut bindings::regulator) -> Self {
        Self(regulator)
    }
}
impl Drop for Regulator {
    fn drop(&mut self) {
        // SAFETY: The pointer is valid by the type invariant.
        unsafe { bindings::regulator_put(self.0) };
    }
}

pub struct Regulators<const COUNT: usize>([bindings::regulator_bulk_data; COUNT], bool);
impl<const COUNT: usize> Regulators<COUNT> {
    pub fn new(
        device: &device::Device,
        supplies: [&CStr; COUNT],
        init_load: Option<[i32; COUNT]>,
    ) -> Result<Self> {
        let mut data = [bindings::regulator_bulk_data::default(); COUNT];
        for i in 0..COUNT {
            data[i].supply = supplies[i].as_char_ptr();
            if let Some(init_load) = init_load {
                data[i].init_load_uA = init_load[i];
            }
        }
        to_result(unsafe {
            bindings::regulator_bulk_get(device.ptr, COUNT as _, data.as_mut_ptr())
        })?;
        Ok(Self(data, false))
    }

    pub fn enable(&mut self) -> Result {
        to_result(unsafe { bindings::regulator_bulk_enable(COUNT as _, self.0.as_mut_ptr()) })?;
        self.1 = true;
        Ok(())
    }

    pub fn disable(&mut self) -> Result {
        to_result(unsafe { bindings::regulator_bulk_disable(COUNT as _, self.0.as_mut_ptr()) })?;
        self.1 = false;
        Ok(())
    }
    pub fn is_enabled(&mut self) -> bool {
        self.1
    }
}

impl<const COUNT: usize> Drop for Regulators<COUNT> {
    fn drop(&mut self) {
        if self.1 {
            if let Err(err) = self.disable() {
                pr_err!(
                    "regulator was enabled in drop and disable errored {:?}",
                    err
                );
            }
        }
        unsafe { bindings::regulator_bulk_free(COUNT as _, self.0.as_mut_ptr()) }
    }
}
