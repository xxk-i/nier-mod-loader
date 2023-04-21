mod hooks;
mod config;

use config::Config;
use config::Plugin;

extern crate core;

use std::collections::HashMap;
use std::error::Error;
use std::{env, mem, ptr, thread, time};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use winapi::shared::minwindef;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, HMODULE, LPVOID, UINT};
use winapi::um::libloaderapi::{GetModuleFileNameA, GetModuleHandleA, GetProcAddress, LoadLibraryA};
use winapi::shared::ntdef::{HRESULT, NULL};
use winapi::um::winnt::LPCSTR;
use glob::{glob};
use winapi::shared::dxgi::IDXGIAdapter;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext};
use winapi::um::d3dcommon::{D3D_DRIVER_TYPE, D3D_FEATURE_LEVEL};

type _D3D11CreateDevice = extern "stdcall" fn(*mut IDXGIAdapter, D3D_DRIVER_TYPE, HMODULE, UINT, *const D3D_FEATURE_LEVEL, UINT, UINT, *mut *mut ID3D11Device, *mut D3D_FEATURE_LEVEL, *mut *mut ID3D11DeviceContext) -> HRESULT;

// cpp naming conventions (i think)
#[allow(non_upper_case_globals)]
static mut dllModule: HINSTANCE = ptr::null_mut();

#[allow(non_upper_case_globals)]
static mut hOriginal: HINSTANCE = ptr::null_mut();

#[allow(non_upper_case_globals)]
static mut pD3D11CreateDevice: Option<_D3D11CreateDevice> = None;

static CONFIG_PATH: &str = ".\\mods\\config.ini";

#[no_mangle]
#[allow(non_snake_case)]
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
            unsafe { dllModule = dll_module };
            let thread_handle = std::thread::spawn(initialize);
        },
        DLL_PROCESS_DETACH => (),
        _ => ()
    }

    minwindef::TRUE
}

// Returns a hashmap of <Filestem, FullPath>
fn get_files_by_glob(pattern: &str) -> Result<HashMap<String, PathBuf>, Box<dyn Error>> {
    let mut map = HashMap::new();

    for entry in glob(pattern)? {
        match entry {
            Ok(path) => {
                println!("Found file at path: {:#?}", path.display());
                map.insert(path.file_stem().unwrap().to_str().unwrap().to_owned(), path);
            }
            Err(e) => println!("{:#?}", e),
        }
    }

    Ok(map)
}

fn get_dlls() -> HashMap<String, PathBuf> {
    println!("Getting dll's");
    get_files_by_glob(".\\mods\\plugins\\**\\*.dll").expect("Failed to search for plugins")
}

fn get_cpks() -> HashMap<String, PathBuf> {
    println!("Getting cpk's");
    get_files_by_glob(".\\mods\\cpks\\**\\*.cpk").expect("Failed to search for cpk's")
}


fn initialize() {

    //Check if we are loaded as d3d11.dll or something else
    //Note: these windows calls suck
    unsafe {
        if cfg!(debug_assertions) {
            AllocConsole();
        }

        // Windows MAX_PATH is supposedly 256 characters, but I am paranoid and don't trust Windows, so we use 0x1000 byte buffer size for a filepath
        let mut filename_buf = [0; 0x1000];
        GetModuleFileNameA(dllModule, filename_buf.as_mut_ptr(), 0x1000);

        let module_filename = CStr::from_ptr(filename_buf.as_mut_ptr()).to_str();

        match module_filename {
            Ok(name) => {
                if name.ends_with("d3d11.dll")
                {
                    println!("Installed as d3d11");
                    hOriginal = LoadLibraryA(CString::new("C:\\Windows\\System32\\d3d11.dll").unwrap().as_ptr());
                    pD3D11CreateDevice = Some(mem::transmute(GetProcAddress(hOriginal, CString::new("D3D11CreateDevice").unwrap().as_ptr())));
                }
            },

            Err(e) => println!("Error resolving module filename... {}... not loading d3d11", e),
        }
    }

    //let the game set the environment to data\\ for us...
    while !env::current_dir().unwrap().ends_with("data") {
        thread::sleep(time::Duration::from_millis(100));
    }

    println!("Initializing...");

    // collect all mod files found in directory
    let dlls: HashMap<String, PathBuf> = get_dlls();
    let cpks: HashMap<String, PathBuf> = get_cpks();

    // load or create config
    let mut config = match Path::new(CONFIG_PATH).exists() {
        true => {
            Config::from(PathBuf::from(CONFIG_PATH))
        },

        false => {
            Config::new(PathBuf::from(CONFIG_PATH))
        }
    };

    let dlls_to_load: Vec<Plugin> = config.parse_dlls(dlls);
    let cpks_to_load: Vec<PathBuf> = config.parse_cpks(cpks);

    config.save();

    if !cpks_to_load.is_empty() {
        unsafe {
            println!("Installing Hooks...");
            hooks::hook_manager::CPK_LIST = Some(cpks_to_load);
            hooks::hook_manager::MODULE_HANDLE = GetModuleHandleA(NULL as LPCSTR);
            match hooks::hook_manager::create_all_hooks() {
                Ok(_) => println!("Hooked successfully!"),
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    if !dlls_to_load.is_empty() {
        let mut early_plugins = Vec::new();
        let mut late_plugins = Vec::new();

        for entry in dlls_to_load {
            match entry {
                Plugin::Early(path) => early_plugins.push(path.clone()),
                Plugin::Late(path) => late_plugins.push(path.clone())
            }
        }
        
        if !early_plugins.is_empty() {
            hooks::hook_manager::load_plugins(&early_plugins);
        }

        if !late_plugins.is_empty() {
            unsafe {
                hooks::hook_manager::DLL_LATE_LOAD_LIST = Some(late_plugins);
            }
        }
    }
}