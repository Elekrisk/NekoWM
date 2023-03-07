use smithay::{wayland::data_device::{DataDeviceHandler, DataDeviceState}, delegate_data_device};

use crate::{drawing_backend::Backend, state::State};



impl<B: Backend> DataDeviceHandler for State<B> {
    fn data_device_state(&self) -> &DataDeviceState {
        todo!()
    }
}

delegate_data_device!(@<B: Backend> State<B>);
