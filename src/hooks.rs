pub mod hook_manager {
    use std::ffi::{c_void, CString};
    use std::{env, mem, ptr};
    use std::os::raw::c_char;
    use std::path::PathBuf;
    use minhook_sys::*;
    use winapi::shared::minwindef::HMODULE;

    #[allow(non_camel_case_types)]
    type fnLoadCPKFromPath = extern "fastcall" fn(i32, *const c_char) -> u64;

    //Hw::cGameContentDeviceSteam_method2(void* arg1, void* arg2)
    #[allow(non_camel_case_types)]
    type fnUnknownFunc = extern "fastcall" fn(u64, u64) -> u32;

    static mut ORIGINAL_UNKNOWN_FUNC: *mut c_void = ptr::null_mut();

    pub static mut MODULE_HANDLE: HMODULE = 0 as HMODULE;
    pub static mut CPK_LIST: Option<Vec<PathBuf>> = None;

    unsafe fn hk_unknown_fn(a1: u64, a2: u64) -> u32 {
        let fn_unknown_func: fnUnknownFunc = mem::transmute(ORIGINAL_UNKNOWN_FUNC);
        let ret = fn_unknown_func(a1, a2);


        let mut current_dir = env::current_dir().unwrap();

        let ptr = (MODULE_HANDLE as u64 + 0x27BE20) as *const ();
        let fn_load_cpk: fnLoadCPKFromPath = std::mem::transmute(ptr);

        for path in CPK_LIST.as_deref().unwrap() {
            println!("Found cpk: {:#}", path.display());

            current_dir.push(path);
            let cstr_path = CString::new(current_dir.as_os_str().to_str().unwrap()).unwrap();
            let const_path = cstr_path.as_ptr() as *const c_char;
            let load_cpk_ret = fn_load_cpk(1, const_path);

            match load_cpk_ret {
                0 => println!("Failure: CPK failed to mount! (LoadCPK returned 0)"),
                1 => println!("Success: CPK mounted! (LoadCPK returned 1)"),
                _ => println!("Unknown: LoadCPK returned {}", load_cpk_ret),
            }
        }

        ret
    }

    pub unsafe fn create_all_hooks() -> Result<(), &'static str> {
        let unknown_func = (MODULE_HANDLE as u64 + 0x281FD0) as *mut c_void;

        MH_Initialize();

        MH_CreateHook(unknown_func, hk_unknown_fn as u64 as *mut c_void , &mut ORIGINAL_UNKNOWN_FUNC);
        MH_EnableHook(unknown_func);


        Ok(())
    }
}