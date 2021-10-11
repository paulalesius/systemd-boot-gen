mod entry;

use alphanumeric_sort;
use dotenv;
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

use entry::Entry;

fn main() {
    dotenv::from_filename("/etc/os-release").expect("Expected /etc/os-release");
    dotenv::from_filename("/etc/default/cmdline").expect("Expected /etc/default/cmdline");
    let machineid = std::fs::read_to_string("/etc/machine-id").expect("Expected /etc/machine-id");
    let machineid = machineid.trim().to_string();

    // Now list available initramfs and vmlinuz
    let re = Regex::new(r"vmlinuz-(.*)").unwrap();
    let readdir = fs::read_dir("/boot").unwrap();
    let mut versions = Vec::new();
    for r in readdir {
        let path = r.unwrap().path().into_os_string();
        let path = path.to_str().unwrap();
        if let Some(capture) = re.captures(path) {
            if capture.len() == 2 {
                let ver = capture.get(1).unwrap().as_str().to_owned();
                if has_initramfs(&ver) {
                    versions.push(ver);
                }
            }
        }
    }

    let osname = env::var("NAME").unwrap();
    let cmdline = env::var("CMDLINE").unwrap();

    alphanumeric_sort::sort_str_slice_rev(&mut versions);
    for version in versions {
        let entry = gen_entry(&machineid, &version, osname.as_str(), cmdline.as_str());
        write_entry(&entry);
    }
}

fn write_entry(entry: &Entry) {
    let outfile = format!("/boot/loader/entries/{}", entry.name);
    let file = File::create(&outfile).expect("Expected to create entry file.");
    let mut writer = BufWriter::new(file);

    write!(writer, "{}\n", entry.title).unwrap();
    write!(writer, "{}\n", entry.version).unwrap();
    write!(writer, "{}\n", entry.linux).unwrap();
    write!(writer, "{}\n", entry.initrd).unwrap();
    write!(writer, "{}\n", entry.options).unwrap();
    println!("Wrote systemd-boot config: {}", outfile);
}

fn has_initramfs(ver: &str) -> bool {
    // Find matching initramfs
    let initramfs = format!("/boot/initramfs-{}.img", ver);
    if Path::new(&initramfs).exists() {
        return true;
    }
    return false;
}

fn gen_entry(machineid: &str, ver: &str, osname: &str, cmdline: &str) -> Entry {
    let name = format!("{}-{}.conf", machineid, ver);
    let title = format!("title {} ({})", osname, ver);
    let version = format!("version {}", ver);
    let linux = format!("linux /vmlinuz-{}", ver);
    let initrd = format!("initrd /initramfs-{}.img", ver);
    let options = format!("options {}", cmdline);

    Entry {
        name,
        title,
        version,
        linux,
        initrd,
        options,
    }
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

        let entry = Entry {
            name: String::from("abcdef-1.0.conf"),
            title: String::from("title test (1.0)"),
            version: String::from("version 1.0"),
            linux: String::from("linux /vmlinuz-1.0"),
            initrd: String::from("initrd /initramfs-1.0.img"),
            options: String::from("options none"),
        };

        let constructed = gen_entry(machineid, version, osname, cmdline);
        assert_eq!(entry, constructed);
    }
}
