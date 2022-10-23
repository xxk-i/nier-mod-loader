mod hooks;
mod config;

extern crate core;

use std::{env, mem, ptr, thread, time};
use std::ffi::CString;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use winapi::shared::minwindef;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, HMODULE, LPVOID, UINT};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use winapi::shared::ntdef::{HRESULT, NULL};
use dll_syringe::{ Syringe, process::OwnedProcess };
use winapi::um::winnt::LPCSTR;
use glob::glob;
use winapi::shared::dxgi::IDXGIAdapter;
use winapi::um::consoleapi;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext};
use winapi::um::d3dcommon::{D3D_DRIVER_TYPE, D3D_FEATURE_LEVEL};

type _D3D11CreateDevice = extern "stdcall" fn(*mut IDXGIAdapter, D3D_DRIVER_TYPE, HMODULE, UINT, *const D3D_FEATURE_LEVEL, UINT, UINT, *mut *mut ID3D11Device, *mut D3D_FEATURE_LEVEL, *mut *mut ID3D11DeviceContext) -> HRESULT;

static mut hOriginal: HINSTANCE = ptr::null_mut();
static mut pD3D11CreateDevice: Option<_D3D11CreateDevice> = None;

#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDevice(pAdapter: *mut IDXGIAdapter,
                                                DriverType: D3D_DRIVER_TYPE,
                                                Software: HMODULE,
                                                Flags: UINT,
                                                pFeatureLevels: *const D3D_FEATURE_LEVEL,
                                                FeatureLevels: UINT,
                                                SDKVersion: UINT,
                                                ppDevice: *mut *mut ID3D11Device,
                                                pFeatureLevel: *mut D3D_FEATURE_LEVEL,
                                                ppImmediateContext: *mut *mut ID3D11DeviceContext) -> HRESULT {
    match pD3D11CreateDevice {
        Some(func) => return func(pAdapter, DriverType, Software, Flags, pFeatureLevels, FeatureLevels, SDKVersion, ppDevice, pFeatureLevel, ppImmediateContext),
        None => panic!("Failed to get original D3D11CreateDevice handle!")
    }
}

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
    unsafe {
        if cfg!(debug_assertions) {
            AllocConsole();
        }
        hOriginal = LoadLibraryA(CString::new("C:\\Windows\\System32\\d3d11.dll").unwrap().as_ptr());
        pD3D11CreateDevice = Some(mem::transmute(GetProcAddress(hOriginal, CString::new("D3D11CreateDevice").unwrap().as_ptr())));
    }

    //let the game set the environment to data\\ for us...
    while !env::current_dir().unwrap().ends_with("data") {
        thread::sleep(time::Duration::from_millis(100));
    }

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