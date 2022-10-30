pub mod hook_manager {
    use std::ffi::{c_void, CString};
    use std::{env, mem, ptr};
    use std::fs::File;
    use std::os::raw::c_char;
    use std::path::PathBuf;
    use minhook_sys::*;
    use winapi::shared::minwindef::HMODULE;

    #[allow(non_camel_case_types)]
    type fnLoadDLCCPKFromPath = extern "fastcall" fn(i32, *const c_char) -> u64;   //this one fucks with us if we try to load 2 extra cpks
    static OFF_LOAD_DLC_CPK_FROM_PATH: u64 = 0x27BE20;

    #[allow(non_camel_case_types)]
    type fnLoadCPKFromPath = extern "fastcall" fn(CString) -> bool;
    static OFF_LOAD_CPK_FROM_PATH: u64 = 0x86AD60;

    //Hw::cGameContentDeviceSteam_method2(void* arg1, void* arg2)
    #[allow(non_camel_case_types)]
    type fnSteamLoadDLCCPK = extern "fastcall" fn(u64, u64) -> u32;
    static OFF_LOAD_STEAM_DLC_CPK: u64 = 0x281FD0;

    static mut ORIGINAL_STEAM_LOAD_DLC_CPK: *mut c_void = ptr::null_mut();

    pub static mut MODULE_HANDLE: HMODULE = 0 as HMODULE;
    pub static mut CPK_LIST: Option<Vec<PathBuf>> = None;
    pub static mut output_file: Option<&mut File> = None;

    unsafe fn hk_steam_load_dlc_cpk(a1: u64, a2: u64) -> u32 {
        let fn_unknown_func: fnSteamLoadDLCCPK = mem::transmute(ORIGINAL_STEAM_LOAD_DLC_CPK);
        let ret = fn_unknown_func(a1, a2);

        let ptr = (MODULE_HANDLE as u64 + OFF_LOAD_CPK_FROM_PATH) as *const ();
        let fn_load_cpk: fnLoadCPKFromPath = mem::transmute(ptr);

        for path in CPK_LIST.as_deref().unwrap() {
            println!("Loading cpk: {:#}", path.display());

            let cstr_path = CString::new(path.with_extension("").to_str().unwrap()).unwrap(); // remove extension
            let load_cpk_ret = fn_load_cpk(cstr_path);

            match load_cpk_ret {
                false => println!("Failure: CPK failed to mount! (LoadCPK returned 0)"),
                true => println!("Success: CPK mounted! (LoadCPK returned 1)"),
                _ => println!("Unknown: LoadCPK returned {}", load_cpk_ret),
            }
        }

        ret
    }

    pub unsafe fn create_all_hooks() -> Result<(), &'static str> {
        let steam_load_dlc_cpk_func = (MODULE_HANDLE as u64 + OFF_LOAD_STEAM_DLC_CPK) as *mut c_void;

        MH_Initialize();

        MH_CreateHook(steam_load_dlc_cpk_func, hk_steam_load_dlc_cpk as u64 as *mut c_void, &mut ORIGINAL_STEAM_LOAD_DLC_CPK);
        MH_EnableHook(steam_load_dlc_cpk_func);


        Ok(())
    }
}