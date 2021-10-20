use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub title: String,
    pub version: String,
    pub linux: String,
    pub initrd: String,
    pub options: String,
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Kernel {
    pub version: String,
    pub config: bool,
    pub vmlinuz: bool,
    pub initramfs: bool,
}

impl Kernel {
    /***
     * The constructor aims to create a kernel object that contains booleans describing
     * if config, vmlinuz, or initramfs, exist as files on the filesystem.
     */
    pub fn new(machineid: &str, version: String) -> Self {
        let config = Path::new(&format!(
            "/boot/loader/entries/{}-{}.conf",
            machineid, version
        ))
        .is_file();
        let vmlinuz = Path::new(&format!("/boot/vmlinuz-{}", version)).is_file();
        let initramfs = Path::new(&format!("/boot/initramfs-{}.img", version)).is_file();
        Kernel {
            version,
            config,
            vmlinuz,
            initramfs,
        }
    }

    /***
     * Check whether both vmlinuz and initramfs files are in place, so that
     * we can proceed with generating the .conf file.
     */
    pub fn is_valid(&self) -> bool {
        self.vmlinuz && self.initramfs
    }
}

pub fn kernel_to_entry(machineid: &str, osname: &str, cmdline: &str, kernel: &Kernel) -> Entry {
    let name = format!("{}-{}.conf", machineid, kernel.version);
    let title = format!("title {} ({})", osname, kernel.version);
    let version = format!("version {}", kernel.version);
    let linux = format!("linux /vmlinuz-{}", kernel.version);
    let initrd = format!("initrd /initramfs-{}.img", kernel.version);
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
