mod hooks;
mod config;

extern crate core;

use std::{env, thread, time};
use std::path::{Path, PathBuf};
use winapi::shared::minwindef;
use winapi::shared::minwindef::{ BOOL, DWORD, HINSTANCE, LPVOID };
use winapi::um::libloaderapi::{ GetModuleHandleA };
use winapi::shared::ntdef::{ NULL };
use dll_syringe::{ Syringe, process::OwnedProcess };
use winapi::um::winnt::LPCSTR;
use glob::glob;
use winapi::um::consoleapi;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
                               dll_module: HINSTANCE,
                               call_reason: DWORD,
                               reserved: LPVOID)
                               -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            let thread_handle = std::thread::spawn(initialize);
        },
        DLL_PROCESS_DETACH => (),
        _ => ()
    }

    minwindef::TRUE
}

fn get_dlls() -> Option<Vec<PathBuf>> {
    let mut dll_list = Vec::new();
    for entry in glob(".\\mods\\plugins\\**\\*.dll").expect("Failed to search for plugins") {
        match entry {
            Ok(path) => {
                println!("Found plugin at path: {:#?}", path.display());
                dll_list.push(path);
            }
            Err(e) => println!("{:#?}", e),
        }
    }

    return if dll_list.is_empty() {
        println!("No DLL's found - does data\\mods\\plugins\\ exist?");
        None
    } else {
        Some(dll_list)
    }
}

fn get_cpks() -> Option<Vec<PathBuf>> {
    println!("Getting cpk's");
    let mut cpk_list = Vec::new();

    for entry in glob(".\\mods\\cpks\\**\\*.cpk").expect("Failed to search for cpk's") {
        match entry {
            Ok(path) => {
                println!("Found mod at path: {:#?}", path.display());
                cpk_list.push(path);
            }
            Err(e) => println!("{:#?}", e),
        }
    }

    return if cpk_list.is_empty() {
        println!("No CPK's found - does data\\mods\\cpks\\ exist?");
        None
    } else {
        Some(cpk_list)
    }
}

fn load_plugins(dll_list: Vec<PathBuf>) {
    let target_process = OwnedProcess::find_first_by_name("NieRAutomata").unwrap();

    let syringe = Syringe::for_process(target_process);

    for path in dll_list {
        match syringe.inject(path) {
            Ok(_) => println!("Successfully injected!"),
            Err(e) => println!("{:#?}", e),
        }
    }
}

fn initialize() {
    //let the game set the environment to data\\ for us...
    while !env::current_dir().unwrap().ends_with("data") {
        thread::sleep(time::Duration::from_millis(100));
    }

    unsafe { consoleapi::AllocConsole(); }
    println!("Initializing...");

    let dll_list: Option<Vec<PathBuf>> = get_dlls();
    let cpk_list: Option<Vec<PathBuf>> = get_cpks();

    if !config::config_manager::config_exists() {
        config::config_manager::create_config(&dll_list, &cpk_list);
    }

    if let Some(cpk_list) = cpk_list {
        let cpk_load_list = config::config_manager::parse_cpk_list(cpk_list);
        if !cpk_load_list.is_empty() {
            unsafe {
                println!("Installing Hooks...");
                hooks::hook_manager::CPK_LIST = Some(cpk_load_list);
                hooks::hook_manager::MODULE_HANDLE = GetModuleHandleA(NULL as LPCSTR);
                match hooks::hook_manager::create_all_hooks() {
                    Ok(_) => println!("Hooked successfully!"),
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
    }

    if let Some(dll_list) = dll_list {
        let dll_load_list = config::config_manager::parse_dll_list(dll_list);
        if !dll_load_list.is_empty() {
            load_plugins(dll_load_list);
        }
    }
}