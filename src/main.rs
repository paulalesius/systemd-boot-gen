mod model;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use model::*;

fn main() {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("microcode")
                .short("u")
                .long("microcode")
                .help("Adds microcode file.")
                .value_name("MICROCODE_FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("remove")
                .short("r")
                .long("remove")
                .help("Remove old configs that lack both vmlinuz and/or initramfs files.")
                .takes_value(false),
        )
        .get_matches();

    dotenv::from_filename("/etc/os-release").expect("Expected /etc/os-release");
    dotenv::from_filename("/etc/default/cmdline").expect("Expected /etc/default/cmdline");
    let machineid = std::fs::read_to_string("/etc/machine-id").expect("Expected /etc/machine-id");
    let machineid = machineid.trim().to_string();
    let osname = env::var("NAME").unwrap();
    let cmdline = env::var("CMDLINE").unwrap();

    let mut kernels = HashSet::new();
    let microcode = check_microcode(args.value_of("microcode"));
    find_kernels(&machineid, &mut kernels);
    find_configs(&machineid, &mut kernels);

    // Now write valid configs that have both vmlinuz and initramfs
    for kernel in &kernels {
        match kernel.is_valid() {
            true => {
                let entry = kernel_to_entry(&machineid, &osname, &cmdline, microcode, kernel);
                write_entry(&entry);
            }
            false => {
                if args.is_present("remove") {
                    let config_path =
                        format!("/boot/loader/entries/{}-{}.conf", machineid, kernel.version);
                    let path = Path::new(&config_path);
                    if path.is_file() {
                        // Config file exists, proceed to remove because it is invalid
                        println!("Removing invalid config: {}", config_path);
                        std::fs::remove_file(path).unwrap();
                    }
                }
            }
        }
    }
}

/***
 * Find existing vmlinuz files and parse versions from the file names,
 * struct Kernel will cosntruct a Kernel object when instantiated.
 */
fn find_kernels(machineid: &str, kernels: &mut HashSet<Kernel>) {
    for entry in fs::read_dir("/boot").expect("Failed to read /boot") {
        let file = entry.unwrap().file_name().to_str().unwrap().to_string();
        if file.starts_with("vmlinuz-") {
            let version = file.strip_prefix("vmlinuz-").unwrap().to_string();
            kernels.insert(Kernel::new(machineid, version));
        }
    }
}

/***
 * Find existing configs and parse kernel versions from their file name,
 * struct Kernel will construct a Kernel object when instantiated.
 */
fn find_configs(machineid: &str, kernels: &mut HashSet<Kernel>) {
    for entry in fs::read_dir("/boot/loader/entries").expect("Failed to read /boot/loader/entry") {
        let file = entry.unwrap().file_name().to_str().unwrap().to_string();
        if file.starts_with(&format!("{}-", machineid)) {
            let version = file
                .strip_prefix(&format!("{}-", machineid))
                .unwrap()
                .strip_suffix(".conf")
                .unwrap()
                .to_string();
            kernels.insert(Kernel::new(machineid, version));
        }
    }
}

fn check_microcode(microcode: Option<&str>) -> Option<&str> {
    if microcode.is_some() {
        let filename = microcode.unwrap().trim();
        let filepath = format!("/boot/{}", filename);

        if Path::new(&filepath).exists() {
            return Some(filename);
        }

        println!("WARNING: {} not found! Ignoring microcode", filepath)
    }

    return None;
}

fn write_entry(entry: &Entry) {
    let outfile = format!("/boot/loader/entries/{}", entry.name);
    let file = File::create(&outfile).expect("Expected to create entry file.");
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{}", entry.title).unwrap();
    writeln!(writer, "{}", entry.version).unwrap();
    writeln!(writer, "{}", entry.linux).unwrap();
    if entry.microcode.is_some() {
        writeln!(writer, "{}", entry.microcode.as_ref().unwrap()).unwrap();
    }
    writeln!(writer, "{}", entry.initrd).unwrap();
    writeln!(writer, "{}", entry.options).unwrap();
    println!("Wrote systemd-boot config: {}", outfile);
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    pub fn test() {
        let machineid = "abcdef";
        let version = "1.0";
        let osname = "test";
        let cmdline = "none";
        let microcode = Some("ucode.img");

        let kernel = Kernel {
            version: version.to_string(),
            config: false,
            initramfs: false,
            vmlinuz: true,
        };

        let expected = Entry {
            name: String::from("abcdef-1.0.conf"),
            title: String::from("title test (1.0)"),
            version: String::from("version 1.0"),
            linux: String::from("linux /vmlinuz-1.0"),
            microcode: Some(String::from("initrd /ucode.img")),
            initrd: String::from("initrd /initramfs-1.0.img"),
            options: String::from("options none"),
        };
        let existing = kernel_to_entry(machineid, osname, cmdline, microcode, &kernel);
        assert_eq!(expected, existing);
    }
}
