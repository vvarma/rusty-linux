#![allow(missing_docs)]

#[cfg(CONFIG_VIDEO_V4L2_I2C)]
use crate::i2c;
use alloc::boxed::Box;

use crate::{bindings, error::from_kernel_result, error::Result, pr_info, PointerWrapper};
use core::{cell::UnsafeCell, marker, pin::Pin};
use macros::vtable;

pub struct SubDev<T: SubDevOps> {
    ptr: UnsafeCell<bindings::v4l2_subdev>,
    ops: marker::PhantomData<T>,
}
unsafe impl<T: SubDevOps> Sync for SubDev<T> {}
unsafe impl<T: SubDevOps> Send for SubDev<T> {}

impl<T: SubDevOps> SubDev<T> {
    #[cfg(CONFIG_VIDEO_V4L2_I2C)]
    pub fn new_i2c(client: &i2c::Client, data: T::Data) -> Result<Pin<Box<Self>>> {
        pr_info!("initializing v4l2 subdev with i2c device\n");
        let subdev = UnsafeCell::new(bindings::v4l2_subdev::default());
        unsafe {
            (*subdev.get()).dev_priv = data.into_pointer() as *mut core::ffi::c_void;
        }
        unsafe {
            bindings::v4l2_i2c_subdev_init(
                subdev.get(),
                client.raw_ptr(),
                SubDevVtable::<T>::build(),
            );
        }
        pr_info!("finished initializing v4l2 subdev with i2c device\n");
        Ok(Pin::from(Box::try_new(Self {
            ptr: subdev,
            ops: marker::PhantomData,
        })?))
    }

    pub fn new(data: T::Data) -> Result<Pin<Box<Self>>> {
        pr_info!("initializing v4l2 subdev\n");
        let subdev = UnsafeCell::new(bindings::v4l2_subdev::default());
        unsafe {
            (*subdev.get()).dev_priv = data.into_pointer() as *mut core::ffi::c_void;
        }

        unsafe {
            bindings::v4l2_subdev_init(subdev.get(), SubDevVtable::<T>::build());
        }
        pr_info!("finished initializing v4l2 subdev\n");
        Ok(Pin::from(Box::try_new(Self {
            ptr: subdev,
            ops: marker::PhantomData,
        })?))
    }
}
impl<T: SubDevOps> Drop for SubDev<T> {
    fn drop(&mut self) {
        pr_info!("clean v4l2 subdev \n");
        unsafe { bindings::v4l2_async_unregister_subdev(self.ptr.get()) }
    }
}

pub trait DataWrapper {
    type Data: PointerWrapper + Send + Sync = ();
}

pub trait SubDevOps:
    CoreSubDevOps
    + TunerSubDevOps
    + VideoSubDevOps
    + VbiSubDevOps
    + AudioSubDevOps
    + IrSubDevOps
    + SensorSubDevOps
    + PadSubDevOps
{
}

struct SubDevVtable<T: SubDevOps>(marker::PhantomData<T>);
impl<T: SubDevOps> SubDevVtable<T> {
    const VTABLE: bindings::v4l2_subdev_ops = bindings::v4l2_subdev_ops {
        core: unsafe { CoreOpsVtable::<T>::build() },
        tuner: unsafe { TunerOpsVtable::<T>::build() },
        audio: unsafe { AudioOpsVtable::<T>::build() },
        video: unsafe { VideoOpsVtable::<T>::build() },
        vbi: unsafe { VbiOpsVtable::<T>::build() },
        ir: unsafe { IrOpsVtable::<T>::build() },
        sensor: unsafe { SensorOpsVtable::<T>::build() },
        pad: unsafe { PadOpsVtable::<T>::build() },
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_ops {
        &Self::VTABLE
    }
}

#[vtable]
pub trait CoreSubDevOps: DataWrapper {
    fn log_status(ctx: <Self::Data as PointerWrapper>::Borrowed<'_>) -> Result;
}

struct CoreOpsVtable<T: CoreSubDevOps>(marker::PhantomData<T>);
impl<T: CoreSubDevOps> CoreOpsVtable<T> {
    unsafe extern "C" fn log_status_callback(sd: *mut bindings::v4l2_subdev) -> core::ffi::c_int {
        let data = unsafe { T::Data::borrow((*sd).dev_priv) };
        from_kernel_result! {
            T::log_status(data)?;
            Ok(0)
        }
    }
    const VTABLE: bindings::v4l2_subdev_core_ops = bindings::v4l2_subdev_core_ops {
        log_status: if T::HAS_LOG_STATUS {
            Some(Self::log_status_callback)
        } else {
            None
        },
        s_io_pin_config: None,
        init: None,
        load_fw: None,
        reset: None,
        s_gpio: None,
        command: None,
        ioctl: None,
        compat_ioctl32: None,
        s_power: None,
        interrupt_service_routine: None,
        subscribe_event: None,
        unsubscribe_event: None,
    };

    const unsafe fn build() -> &'static bindings::v4l2_subdev_core_ops {
        &Self::VTABLE
    }
}

#[vtable]
pub trait TunerSubDevOps {}

struct TunerOpsVtable<T: TunerSubDevOps>(marker::PhantomData<T>);
impl<T: TunerSubDevOps> TunerOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_tuner_ops = bindings::v4l2_subdev_tuner_ops {
        standby: None,
        s_radio: None,
        s_frequency: None,
        g_frequency: None,
        enum_freq_bands: None,
        g_tuner: None,
        s_tuner: None,
        g_modulator: None,
        s_modulator: None,
        s_type_addr: None,
        s_config: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_tuner_ops {
        &Self::VTABLE
    }
}

#[vtable]
pub trait AudioSubDevOps {}
struct AudioOpsVtable<T: AudioSubDevOps>(marker::PhantomData<T>);
impl<T: AudioSubDevOps> AudioOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_audio_ops = bindings::v4l2_subdev_audio_ops {
        s_clock_freq: None,
        s_i2s_clock_freq: None,
        s_routing: None,
        s_stream: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_audio_ops {
        &Self::VTABLE
    }
}
#[vtable]
pub trait VideoSubDevOps {}
struct VideoOpsVtable<T: VideoSubDevOps>(marker::PhantomData<T>);
impl<T: VideoSubDevOps> VideoOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_video_ops = bindings::v4l2_subdev_video_ops {
        s_routing: None,
        s_crystal_freq: None,
        g_std: None,
        s_std: None,
        s_std_output: None,
        g_std_output: None,
        querystd: None,
        g_tvnorms: None,
        g_tvnorms_output: None,
        g_input_status: None,
        s_stream: None,
        g_pixelaspect: None,
        g_frame_interval: None,
        s_frame_interval: None,
        s_dv_timings: None,
        g_dv_timings: None,
        query_dv_timings: None,
        s_rx_buffer: None,
        pre_streamon: None,
        post_streamoff: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_video_ops {
        &Self::VTABLE
    }
}
#[vtable]
pub trait VbiSubDevOps {}
struct VbiOpsVtable<T: VbiSubDevOps>(marker::PhantomData<T>);
impl<T: VbiSubDevOps> VbiOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_vbi_ops = bindings::v4l2_subdev_vbi_ops {
        decode_vbi_line: None,
        s_vbi_data: None,
        g_vbi_data: None,
        g_sliced_vbi_cap: None,
        s_raw_fmt: None,
        g_sliced_fmt: None,
        s_sliced_fmt: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_vbi_ops {
        &Self::VTABLE
    }
}
#[vtable]
pub trait IrSubDevOps {}
struct IrOpsVtable<T: IrSubDevOps>(marker::PhantomData<T>);
impl<T: IrSubDevOps> IrOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_ir_ops = bindings::v4l2_subdev_ir_ops {
        rx_read: None,
        rx_g_parameters: None,
        rx_s_parameters: None,
        tx_write: None,
        tx_g_parameters: None,
        tx_s_parameters: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_ir_ops {
        &Self::VTABLE
    }
}
#[vtable]
pub trait SensorSubDevOps {}
struct SensorOpsVtable<T: SensorSubDevOps>(marker::PhantomData<T>);
impl<T: SensorSubDevOps> SensorOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_sensor_ops = bindings::v4l2_subdev_sensor_ops {
        g_skip_top_lines: None,
        g_skip_frames: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_sensor_ops {
        &Self::VTABLE
    }
}
#[vtable]
pub trait PadSubDevOps {}
struct PadOpsVtable<T: PadSubDevOps>(marker::PhantomData<T>);
impl<T: PadSubDevOps> PadOpsVtable<T> {
    const VTABLE: bindings::v4l2_subdev_pad_ops = bindings::v4l2_subdev_pad_ops {
        init_cfg: None,
        enum_mbus_code: None,
        enum_frame_size: None,
        enum_frame_interval: None,
        get_fmt: None,
        set_fmt: None,
        get_selection: None,
        set_selection: None,
        get_edid: None,
        set_edid: None,
        dv_timings_cap: None,
        enum_dv_timings: None,
        get_frame_desc: None,
        set_frame_desc: None,
        get_mbus_config: None,
        #[cfg(CONFIG_MEDIA_CONTROLLER)]
        link_validate: None,
    };
    const unsafe fn build() -> &'static bindings::v4l2_subdev_pad_ops {
        &Self::VTABLE
    }
}

