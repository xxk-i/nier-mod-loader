pub mod hook_manager {
    use dll_syringe::Syringe;
    use dll_syringe::process::OwnedProcess;
    use retour::static_detour;
    use std::ffi::{CString};
    use std::{mem};
    use std::error::Error;
    use std::path::PathBuf;
    use winapi::shared::minwindef::HMODULE;

    static_detour! {
        static SteamLoadDLCCPKHook: unsafe extern "fastcall" fn(u64, u64) -> u32;
    }

    // #[allow(non_camel_case_types)]
    // type FnLoadDLCCPKFromPath = extern "fastcall" fn(i32, *const c_char) -> u64;   //this one fucks with us if we try to load 2 extra cpks
    // static OFF_LOAD_DLC_CPK_FROM_PATH: u64 = 0x27BE20;

    #[allow(non_camel_case_types)]
    type FnLoadCPKFromPath = extern "fastcall" fn(CString) -> bool;
    static OFF_LOAD_CPK_FROM_PATH: u64 = 0x86AD60;

    //Hw::cGameContentDeviceSteam_method2(void* arg1, void* arg2)
    #[allow(non_camel_case_types)]
    type FnSteamLoadDLCCPK = extern "fastcall" fn(u64, u64) -> u32;
    static OFF_LOAD_STEAM_DLC_CPK: u64 = 0x281FD0;

    // static mut ORIGINAL_STEAM_LOAD_DLC_CPK: *mut c_void = ptr::null_mut();

    pub static mut MODULE_HANDLE: HMODULE = 0 as HMODULE;
    pub static mut CPK_LIST: Option<Vec<PathBuf>> = None; // is guaranteed to be Some(_) when hooks are called by lib.rs::initialize()
    pub static mut DLL_LATE_LOAD_LIST: Option<Vec<PathBuf>> = None;
    // pub static mut output_file: Option<&mut File> = None;

    // this hooks the function that loads DLC cpk's at main menu, and loads our own afterwards
    // it also loads Plugin::Late(_)'s
    fn hk_steam_load_dlc_cpk(a1: u64, a2: u64) -> u32 {
        unsafe {
            //let fn_unknown_func: FnSteamLoadDLCCPK = mem::transmute(ORIGINAL_STEAM_LOAD_DLC_CPK);
            let ret = SteamLoadDLCCPKHook.call(a1, a2);

            let ptr = (MODULE_HANDLE as u64 + OFF_LOAD_CPK_FROM_PATH) as *const ();
            let fn_load_cpk: FnLoadCPKFromPath = mem::transmute(ptr);

            // load our cpks
            for path in CPK_LIST.as_deref().unwrap() {  // as_deref because we cannot unwrap in place (CPK_LIST is static)
                println!("Loading cpk: {:#}", path.display());

                let cstr_path = CString::new(path.with_extension("").to_str().unwrap()).unwrap(); // remove extension
                let load_cpk_ret = fn_load_cpk(cstr_path);

                match load_cpk_ret {
                    false => println!("Failure: CPK failed to mount! (LoadCPK returned 0)"),
                    true => println!("Success: CPK mounted! (LoadCPK returned 1)"),

                    // allowing unreachable because not sure if function could throw a non 0/1
                    #[allow(unreachable_patterns)]
                    _ => println!("Unknown: LoadCPK returned {}", load_cpk_ret),
                }
            }

            if let Some(dll_list) = &DLL_LATE_LOAD_LIST {
                println!("Late loading plugins");
                load_plugins(dll_list);
            }

            ret
        }
    }

    pub unsafe fn create_all_hooks() -> Result<(), Box<dyn Error>> {
        let addr_steam_load_dlc_cpk = MODULE_HANDLE as u64 + OFF_LOAD_STEAM_DLC_CPK;

        //Back to mem::transmute, slight stink but its in the detour-rs docs and ultimately theres no getting around shit like this
        let target_steam_load: FnSteamLoadDLCCPK = mem::transmute(addr_steam_load_dlc_cpk);

        SteamLoadDLCCPKHook
            .initialize(target_steam_load, hk_steam_load_dlc_cpk )?
            .enable()?;

        Ok(())
    }

    pub fn load_plugins(dll_list: &Vec<PathBuf>) {
        let target_process = OwnedProcess::find_first_by_name("NieRAutomata").unwrap();

        let syringe = Syringe::for_process(target_process);

        for path in dll_list {
            match syringe.inject(&path) {
                Ok(_) => println!("Successfully injected {:?}!", path.file_stem().unwrap()),
                Err(e) => println!("{:#?}", e),
            }
        }
}
}