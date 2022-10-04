pub mod config_manager {
    use std::path::{Path, PathBuf};
    use ini::Ini;

    static CONFIG_PATH: &str = ".\\mods\\config.ini";

    pub fn config_exists() -> bool {
        match Path::new(CONFIG_PATH).try_exists() {
            Ok(b) => return b,
            Err(e) => panic!("Error checking for config.ini: {}", e),
        };
    }

    pub fn create_config(dll_list: &Option<Vec<PathBuf>>, cpk_list: &Option<Vec<PathBuf>>) {
        let mut conf = Ini::new();

        if let Some(dll_list) = dll_list {
            for dll in dll_list {
                add_dll_entry(&mut conf, dll);
            }
        } else {
            conf.with_section(Some("DLL"));
        }

        if let Some(cpk_list) = cpk_list {
            for cpk in cpk_list {
                add_cpk_entry(&mut conf, cpk);
            }
        } else {
            conf.with_section(Some("DLL"));
        }

        conf.write_to_file(CONFIG_PATH).unwrap_or_else(|err| panic!("Error writing config file: {}", err));
    }

    pub fn parse_dll_list(dll_list: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut i = Ini::load_from_file(CONFIG_PATH).unwrap_or_else(|err| panic!("Error reading config file! {}", err));

        let mut new_list: Vec<PathBuf> = Vec::new();

        'outer: for dll in dll_list {
            for (sec, prop) in i.iter_mut() {
                if sec.unwrap().eq("DLL") {
                    for (k, v) in prop.iter() {
                        if k.eq(dll.file_stem().unwrap().to_str().unwrap()) {
                            if v.eq("TRUE") {
                                new_list.push(dll.clone());
                            }

                            else if !v.eq("FALSE") {
                                println!("Config entry invalid? {}={}", k, v);
                            }

                            continue 'outer;
                        }
                    }
                }
            }

            add_dll_entry(&mut i, &dll);
            i.write_to_file(CONFIG_PATH);
        }

        new_list
    }

    pub fn parse_cpk_list(cpk_list: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut i = Ini::load_from_file(CONFIG_PATH).unwrap_or_else(|err| panic!("Error reading config file: {}", err));

        let mut new_list: Vec<PathBuf> = Vec::new();

        'outer: for cpk in cpk_list {
            for (sec, prop) in i.iter_mut() {
                if sec.unwrap().eq("CPK") {
                    for (k, v) in prop.iter() {
                        if k.eq(cpk.file_stem().unwrap().to_str().unwrap()) {
                            if v.eq("TRUE") {
                                new_list.push(cpk.clone());
                            }

                            else if !v.eq("FALSE") {
                                println!("Config entry invalid? {}={}", k, v);
                            }

                            continue 'outer;
                        }
                    }
                }
            }

            add_cpk_entry(&mut i, &cpk);
            i.write_to_file(CONFIG_PATH);
        }

        new_list
    }

    fn add_dll_entry(ini: &mut Ini, dll: &PathBuf) {
        ini.with_section(Some("DLL"))
            .set(dll.file_stem().unwrap().to_str().unwrap(), "TRUE");
    }

    fn add_cpk_entry(ini: &mut Ini, cpk: &PathBuf) {
        ini.with_section(Some("CPK"))
            .set(cpk.file_stem().unwrap().to_str().unwrap(), "TRUE");
    }
}