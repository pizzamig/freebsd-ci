use crate::pot::{
    destroy_pot, get_pot_path, is_pot_present, revert_fscomp, spawn_builder_pot, PotError,
};
use crate::{BuildJob, Opt, Project};
use failure::{Error, Fail};
use log::debug;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Fail)]
pub(crate) enum BuildError {
    #[fail(display = "Missing pot: {}", potname)]
    PotNotPresent { potname: String },
}

fn generate_build_script(pot_name: &str, _job: &BuildJob) -> Result<(), Error> {
    let pot_path = get_pot_path(pot_name)?;
    let mut file_path = PathBuf::from(&pot_path);
    file_path.push("m");
    file_path.push("root");
    file_path.push("build.sh");
    println!("Creating the file {:?}", file_path);
    let mut f = File::create(&file_path)?;
    let metadata = f.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    f.set_permissions(permissions)?;
    // shebang
    writeln!(f, "#!/bin/sh\n")?;
    // environment
    writeln!(f, "export HOME=/root\n")?;
    writeln!(f, "export PATH=/root/.cargo/bin:/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin\n")?;

    // build script
    writeln!(f, "cd /mnt\n")?;
    writeln!(f, "cargo clippy --release\n")?;
    writeln!(f, "cargo build --release\n")?;
    writeln!(f, "cargo test --release\n")?;
    Ok(())
}

fn run_build_script(pot_name: &str) -> Result<(), Error> {
    let output = Command::new("pot")
        .args(&["set-cmd", "-p", &pot_name, "-c", "/root/build.sh"])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::PotStartFailed {
            name: pot_name.to_string(),
        }));
    }

    let output = Command::new("pot").args(&["start", &pot_name]).output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::PotStartFailed {
            name: pot_name.to_string(),
        }));
    }
    // write the log somewhere
    let mut log_filename = pot_name.to_string();
    log_filename.push_str(".log");
    let mut log_file = File::create(&log_filename)?;
    log_file.write_all(&output.stdout)?;

    let mut logerr_filename = pot_name.to_string();
    logerr_filename.push_str("_err.log");
    let mut logerr_file = File::create(&logerr_filename)?;
    logerr_file.write_all(&output.stderr)?;
    Ok(())
}

pub(crate) fn build(queue: &[BuildJob], prj: &Project, config: &Opt) -> Result<(), Error> {
    for b in queue {
        let image_name = b.to_string();
        let fscomp_name = prj.to_string();
        if !is_pot_present(&image_name) {
            return Err(Error::from(BuildError::PotNotPresent {
                potname: image_name,
            }));
        }

        // spawn the container
        let pot_name = spawn_builder_pot(&image_name, &fscomp_name, &config)?;
        println!("Spawned new pot: {}", pot_name);
        // run the build
        generate_build_script(&pot_name, b)?;
        run_build_script(&pot_name)?;
        // collect the results
        // cleanup
        // // destroy the pot
        destroy_pot(&pot_name)?;
        debug!("Destroyed pot: {}", pot_name);
        // // revert the fscomp
        revert_fscomp(&fscomp_name)?;
        debug!("Revert fscomp : {}", fscomp_name);
    }
    Ok(())
}
