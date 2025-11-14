use xcb::x::{Atom, GetAtomName};
use xcb::{
    randr::GetMonitors,
    Connection, Xid,
};

use crate::os_level::MonitorInfo;

fn get_name(conn: &Connection, atom: Atom) -> Result<String, String> {
    let get_atom_value = conn.send_request(&GetAtomName { atom });

    let get_atom_value_reply = conn.wait_for_reply(get_atom_value);
    if get_atom_value_reply.is_err() {
        return Err(get_atom_value_reply.err().unwrap().to_string());
    }
    Ok(get_atom_value_reply
        .expect("get_atom_value_reply")
        .name()
        .to_string())
}

pub fn get_all() -> Result<Vec<MonitorInfo>, String> {
    let (conn, index) = match Connection::connect(None) {
        Ok(x) => x,
        Err(x) => return Err(x.to_string()),
    };

    let setup = conn.get_setup();

    let screen = setup.roots().nth(index as usize).unwrap();

    let get_monitors_cookie = conn.send_request(&GetMonitors {
        window: screen.root(),
        get_active: true,
    });

    let get_monitors_reply = match conn.wait_for_reply(get_monitors_cookie) {
        Ok(x) => x,
        Err(x) => return Err(x.to_string()),
    };

    let monitor_info_iterator = get_monitors_reply.monitors();

    let mut display_infos = Vec::new();

    for monitor_info in monitor_info_iterator {
        let output = monitor_info.outputs().first().unwrap();

        let name = get_name(&conn, monitor_info.name())?;

        display_infos.push(MonitorInfo {
            id: output.resource_id() as usize,
            name,
            width: monitor_info.width() as usize,
            height: monitor_info.height() as usize,
        });
    }

    Ok(display_infos)
}
