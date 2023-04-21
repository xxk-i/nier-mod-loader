use std::path::{PathBuf};
use std::collections::HashMap;
use ini::Ini;

pub enum Plugin {
    Early(PathBuf),
    Late(PathBuf)
}

// wrapper around ini::Ini that makes life easier for nier-mod-loader
pub struct Config {
    ini: Ini,
    path: PathBuf
}

impl Config {

    // creates a new config and saves it to the given path
    pub fn new(path: PathBuf) -> Config {
        let mut config = Config {
            ini: Ini::new(),
            path
        };

        // little meme to create empty section
        config.ini.entry(Some("DLL".to_string())).or_insert(Default::default());
        config.ini.entry(Some("CPK".to_string())).or_insert(Default::default());

        config.save();

        config
    }

    pub fn from(path: PathBuf) -> Config {
        Config {
            ini: Ini::load_from_file(&path).expect("failed to load config at path"),
            path: path
        }
    }

    // parse_cpks takes a collection of files, and returns the collection of cpk's to load
    pub fn parse_cpks(&mut self, mut cpks: HashMap<String, PathBuf>) -> Vec<PathBuf> {
        if cpks.is_empty() {
            return Vec::new()
        }

        // our input cpks could have entries that don't yet exst in our config, so first we filter out the ones that do
        let cpk_section = self.ini.section(Some("CPK")).unwrap();

        let mut cpks_to_load: Vec<PathBuf> = Vec::new();
        
        // match config to entry in cpks
        // if it matches, move the entry out of cpks into cpks_to_load
        for (key, value) in cpk_section.iter() {
            if let Some(_) = cpks.get(key) {
                if value.eq("TRUE") {
                    cpks_to_load.push(cpks.remove(key).unwrap());
                }
            }
        }

        // now add the leftovers in cpks to config,
        // and (load new mods by default) move them to cpks_to_load
        for entry in cpks.iter_mut() {
            self.add_cpk_entry(entry.0);
            cpks_to_load.push(entry.1.clone());
        }

        cpks_to_load
    }

    pub fn parse_dlls(&mut self, mut dlls: HashMap<String, PathBuf>) -> Vec<Plugin> {
        if dlls.is_empty() {
            return Vec::new();
        }

        let dll_section = self.ini.section(Some("DLL")).unwrap();

        let mut dlls_to_load: Vec<Plugin> = Vec::new();

        for (key, value) in dll_section.iter() {
            if let Some(_) = dlls.get(key) {
                if value.eq("EARLY") {
                    dlls_to_load.push(Plugin::Early(dlls.remove(key).unwrap()));
                }

                else if value.eq("LATE") {
                    dlls_to_load.push(Plugin::Late(dlls.remove(key).unwrap()));
                }
            }
        }

        for entry in dlls.iter_mut() {
            self.add_dll_entry(entry.0);
            dlls_to_load.push(Plugin::Early(entry.1.clone()));
        }

        dlls_to_load
    }

    // default load option is EARLY (prob shouldn't but most plugins at the moment assume inject on launch)
    fn add_dll_entry(&mut self, name: &String) {
        self.ini.with_section(Some("DLL"))
            .set(name, "EARLY");
    }

    fn add_cpk_entry(&mut self, name: &String) {
        self.ini.with_section(Some("CPK"))
            .set(name, "TRUE");
    }

    pub fn save(&self) {
        self.ini.write_to_file(&self.path).unwrap();
    }
}