#![allow(missing_docs)]
use core::{cell::UnsafeCell, slice};

use alloc::vec::Vec;

use crate::{bindings, fwnode::Node, ARef, to_result, Result};

#[derive(Debug)]
pub struct FwnodeEndpoint {
    endpoint: UnsafeCell<bindings::v4l2_fwnode_endpoint>,
    pub bus: Bus,
    pub link_frequencies: Vec<u64>,
}

impl FwnodeEndpoint {
    fn get_bus(bus_type: BusType, endpoint: &UnsafeCell<bindings::v4l2_fwnode_endpoint>) -> Bus {
        match bus_type {
            BusType::Unknown => Bus::Other,
            BusType::Parallel => unsafe {
                Bus::Parallel {
                    flags: (*endpoint.get()).bus.parallel.flags,
                    bus_width: (*endpoint.get()).bus.parallel.bus_width,
                    data_shift: (*endpoint.get()).bus.parallel.data_shift,
                }
            },
            BusType::Bt656 => Bus::Other,
            BusType::Csi1 => unsafe {
                Bus::Csi1 {
                    clock_inv: (*endpoint.get()).bus.mipi_csi1.clock_inv() != 0,
                    strobe: (*endpoint.get()).bus.mipi_csi1.strobe() != 0,
                    lane_polarity: (*endpoint.get()).bus.mipi_csi1.lane_polarity,
                    data_lane: (*endpoint.get()).bus.mipi_csi1.data_lane,
                    clock_lane: (*endpoint.get()).bus.mipi_csi1.clock_lane,
                }
            },
            BusType::Ccp2 => unsafe {
                Bus::Ccp2 {
                    clock_inv: (*endpoint.get()).bus.mipi_csi1.clock_inv() != 0,
                    strobe: (*endpoint.get()).bus.mipi_csi1.strobe() != 0,
                    lane_polarity: (*endpoint.get()).bus.mipi_csi1.lane_polarity,
                    data_lane: (*endpoint.get()).bus.mipi_csi1.data_lane,
                    clock_lane: (*endpoint.get()).bus.mipi_csi1.clock_lane,
                }
            },
            BusType::Csi2Dphy => unsafe {
                Bus::Csi2Dphy {
                    flags: (*endpoint.get()).bus.mipi_csi2.flags,
                    data_lanes: (*endpoint.get()).bus.mipi_csi2.data_lanes,
                    clock_lane: (*endpoint.get()).bus.mipi_csi2.clock_lane,
                    num_data_lines: (*endpoint.get()).bus.mipi_csi2.num_data_lanes,
                    lane_polarities: (*endpoint.get()).bus.mipi_csi2.lane_polarities,
                }
            },
            BusType::Csi2Cphy => unsafe {
                Bus::Csi2Cphy {
                    flags: (*endpoint.get()).bus.mipi_csi2.flags,
                    data_lanes: (*endpoint.get()).bus.mipi_csi2.data_lanes,
                    clock_lane: (*endpoint.get()).bus.mipi_csi2.clock_lane,
                    num_data_lines: (*endpoint.get()).bus.mipi_csi2.num_data_lanes,
                    lane_polarities: (*endpoint.get()).bus.mipi_csi2.lane_polarities,
                }
            },
            BusType::Dpi => Bus::Other,
            BusType::Invalid => Bus::Invalid,
        }
    }
    pub fn from(node: ARef<Node>, bus_type: BusType) -> Result<Self> {
        let mut endpoint = bindings::v4l2_fwnode_endpoint::default();
        endpoint.bus_type = bus_type as u32;
        let endpoint = UnsafeCell::new(endpoint);
        to_result(unsafe {
            bindings::v4l2_fwnode_endpoint_alloc_parse(node.0.get(), endpoint.get())
        })?;
        let bus = Self::get_bus(bus_type, &endpoint);
        let lf = unsafe {
            slice::from_raw_parts(
                (*endpoint.get()).link_frequencies,
                (*endpoint.get()).nr_of_link_frequencies as usize,
            )
            .try_to_vec()?
        };
        Ok(Self {
            endpoint,
            bus,
            link_frequencies: lf,
        })
    }
}
impl Drop for FwnodeEndpoint {
    fn drop(&mut self) {
        unsafe { bindings::v4l2_fwnode_endpoint_free(self.endpoint.get()) }
    }
}

#[derive(Debug)]
pub enum Bus {
    Other,
    Parallel {
        flags: u32,
        bus_width: u8,
        data_shift: u8,
    },
    Csi1 {
        clock_inv: bool,
        strobe: bool,
        lane_polarity: [bool; 2],
        data_lane: u8,
        clock_lane: u8,
    },
    Ccp2 {
        clock_inv: bool,
        strobe: bool,
        lane_polarity: [bool; 2],
        data_lane: u8,
        clock_lane: u8,
    },
    Csi2Dphy {
        flags: u32,
        data_lanes: [u8; bindings::V4L2_MBUS_CSI2_MAX_DATA_LANES as _],
        clock_lane: u8,
        num_data_lines: u8,
        lane_polarities: [bool; 1 + bindings::V4L2_MBUS_CSI2_MAX_DATA_LANES as usize],
    },
    Csi2Cphy {
        flags: u32,
        data_lanes: [u8; bindings::V4L2_MBUS_CSI2_MAX_DATA_LANES as _],
        clock_lane: u8,
        num_data_lines: u8,
        lane_polarities: [bool; 1 + bindings::V4L2_MBUS_CSI2_MAX_DATA_LANES as usize],
    },
    Invalid,
}

#[derive(Clone, Copy)]
pub enum BusType {
    Unknown = bindings::v4l2_mbus_type_V4L2_MBUS_UNKNOWN as _,
    Parallel = bindings::v4l2_mbus_type_V4L2_MBUS_PARALLEL as _,
    Bt656 = bindings::v4l2_mbus_type_V4L2_MBUS_BT656 as _,
    Csi1 = bindings::v4l2_mbus_type_V4L2_MBUS_CSI1 as _,
    Ccp2 = bindings::v4l2_mbus_type_V4L2_MBUS_CCP2 as _,
    Csi2Dphy = bindings::v4l2_mbus_type_V4L2_MBUS_CSI2_DPHY as _,
    Csi2Cphy = bindings::v4l2_mbus_type_V4L2_MBUS_CSI2_CPHY as _,
    Dpi = bindings::v4l2_mbus_type_V4L2_MBUS_DPI as _,
    Invalid = bindings::v4l2_mbus_type_V4L2_MBUS_INVALID as _,
}
