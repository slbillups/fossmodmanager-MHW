use tauri_plugin_os::OsExt;

fn main() {
    let os_info = tauri_plugin_os::os().version();
    println!("OS Info: {:?}", os_info);
}

    //todo: add osinfo to modregistry
