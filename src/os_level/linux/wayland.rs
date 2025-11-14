use smithay_client_toolkit::output::{OutputHandler, OutputInfo, OutputState};
use smithay_client_toolkit::reexports::client::globals::registry_queue_init;
use smithay_client_toolkit::reexports::client::protocol::wl_output;
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::{delegate_output, delegate_registry, registry_handlers};

use crate::os_level::MonitorInfo;

impl From<&OutputInfo> for MonitorInfo {
    fn from(info: &OutputInfo) -> Self {
        let (w, h) = info.logical_size.unwrap_or(info.physical_size);
        MonitorInfo {
            id: info.id as usize,
            name: info.name.clone().unwrap_or_default(),
            width: w as usize,
            height: h as usize,
        }
    }
}

/// Application data.
struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);

impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        OutputState,
    }
}

pub fn get_all() -> Result<Vec<MonitorInfo>, String> {
    let conn = match Connection::connect_to_env() {
        Ok(x) => x,
        Err(x) => return Err(x.to_string()),
    };

    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);

    let output_delegate = OutputState::new(&globals, &qh);

    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
    };

    event_queue.roundtrip(&mut list_outputs).unwrap();

    list_outputs
        .output_state
        .outputs()
        .map(|output| {
            list_outputs
                .output_state
                .info(&output)
                .map(|o| MonitorInfo::from(&o))
                .ok_or(String::from("Cannot get info from Output in Wayland"))
        })
        .collect::<Result<Vec<MonitorInfo>, String>>()
}
