#![allow(missing_docs)]
use crate::{
    bindings, device, driver, error::from_kernel_result, of, str::CStr, to_result,
    types::PointerWrapper, Result, ThisModule,
};
pub trait Driver {
    /// Data stored on client by driver.
    ///
    /// todo
    /// Require that `Data` implements `PointerWrapper`. We guarantee to
    /// never move the underlying wrapped data structure. This allows
    type Data: PointerWrapper + Send + Sync + driver::DeviceRemoval = ();
    /// The type holding information about each device id supported by the driver.
    type IdInfo: 'static = ();

    /// The table of device ids supported by the driver.
    const OF_DEVICE_ID_TABLE: Option<driver::IdTable<'static, of::DeviceId, Self::IdInfo>> = None;
    fn probe(client: &Client) -> Result<Self::Data>;
    fn remove(_data: &Self::Data) {}
}

/// A registration of a platform driver.
pub type Registration<T> = driver::Registration<Adapter<T>>;

/// An adapter for the registration of platform drivers.
pub struct Adapter<T: Driver>(T);

impl<T: Driver> driver::DriverOps for Adapter<T> {
    type RegType = bindings::i2c_driver;

    unsafe fn register(
        reg: *mut bindings::i2c_driver,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        // SAFETY: By the safety requirements of this function (defined in the trait definition),
        // `reg` is non-null and valid.
        let i2cdrv = unsafe { &mut *reg };
        i2cdrv.driver.name = name.as_char_ptr();
        i2cdrv.probe_new = Some(Self::probe_callback);
        i2cdrv.remove = Some(Self::remove_callback);
        if let Some(t) = T::OF_DEVICE_ID_TABLE {
            i2cdrv.driver.of_match_table = t.as_ref();
        }
        to_result(unsafe { bindings::i2c_register_driver(module.0, reg) })
    }

    unsafe fn unregister(reg: *mut bindings::i2c_driver) {
        unsafe { bindings::i2c_del_driver(reg) };
    }
}

impl<T: Driver> Adapter<T> {
    extern "C" fn probe_callback(i2cdev: *mut bindings::i2c_client) -> core::ffi::c_int {
        from_kernel_result! {
        let client = unsafe {Client::from_ptr(i2cdev)};
        let data = T::probe(&client)?;
        unsafe {(*i2cdev).dev.driver_data= data.into_pointer() as _};
        Ok(0)
        }
    }
    extern "C" fn remove_callback(i2cdev: *mut bindings::i2c_client) {
        let ptr = unsafe { bindings::dev_get_drvdata(&mut (*i2cdev).dev) };
        let data = unsafe { T::Data::from_pointer(ptr) };
        T::remove(&data);
        <T::Data as driver::DeviceRemoval>::device_remove(&data);
    }
}

pub struct Client {
    ptr: *mut bindings::i2c_client,
}
impl Client {
    unsafe fn from_ptr(ptr: *mut bindings::i2c_client) -> Self {
        Self { ptr }
    }
    pub unsafe fn raw_ptr(&self) -> *mut bindings::i2c_client {
        self.ptr
    }
}
unsafe impl device::RawDevice for Client {
    fn raw_device(&self) -> *mut bindings::device {
        unsafe { &mut (*self.ptr).dev }
    }
}

/// Declares a kernel module that exposes a single i2c driver.
///
#[macro_export]
macro_rules! module_i2c_driver {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::i2c::Adapter<T>, { $($f)* });
    };
}
