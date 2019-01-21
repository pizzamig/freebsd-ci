use crate::github::RepoStatus;
use crate::Opt;
use crate::Project;
use failure::{Error, Fail};
use std::process::Command;
use std::{thread, time};

pub(crate) fn is_pot_available() -> bool {
    true
}

#[derive(Debug, Fail)]
pub(crate) enum PotError {
    #[fail(display = "Not able to get the fscomp prefix")]
    FscompPrefix,
    #[fail(display = "Not able to get the pot prefix")]
    PotPrefix,
    #[fail(display = "Not able to get the fscomp list")]
    FscompList,
    #[fail(display = "Fscomp {} already present", name)]
    FscompAlreadyPresent { name: String },
    #[fail(display = "Fscomp create failed on {}", name)]
    FscompCreateFailed { name: String },
    #[fail(display = "Fscomp snapshot failed on {}@{}", name, snap)]
    FscompSnapshotFailed { name: String, snap: String },
    #[fail(display = "Fscomp revert failed on {}@{}", name, snap)]
    FscompRevertFailed { name: String, snap: String },
    #[fail(display = "Fscomp destroy failed on {}", name)]
    FscompDestroyFailed { name: String },
    #[fail(display = "Pot {} already present", name)]
    PotAlreadyPresent { name: String },
    #[fail(display = "Pot destroy failed on {}", name)]
    PotDestroyFailed { name: String },
    #[fail(display = "Pot start failed on {}", name)]
    PotStartFailed { name: String },
    #[fail(display = "Pot stop failed on {}", name)]
    PotStopFailed { name: String },
    #[fail(display = "Pot clone failed on {} from parent {}", name, parent)]
    PotCloneFailed { name: String, parent: String },
    #[fail(display = "Add fscomp {} to pot {} at {} failed", fscomp, pot, mnt)]
    AddFscompFailed {
        pot: String,
        fscomp: String,
        mnt: String,
    },
    #[fail(display = "Git clone failed from {} to {}", url, path)]
    GitCloneFailed {
        url: String,
        path: String,
        stderr: String,
    },
    #[fail(display = "Git clone failed from {} to {} with tag {}", url, path, tag)]
    GitCloneTagFailed {
        url: String,
        path: String,
        tag: String,
        stderr: String,
    },
}

fn _is_fscomp_present(fscompname: &str) -> Result<bool, Error> {
    let output = Command::new("pot").args(&["ls", "-fq"]).output()?;
    if output.status.success() {
        let output_str = String::from_utf8(output.stdout)?;
        if output_str.lines().any(|x| x == fscompname) {
            return Ok(true);
        }
    } else {
        return Err(Error::from(PotError::FscompList));
    }
    Ok(false)
}

pub(crate) fn is_fscomp_present(potname: &str) -> bool {
    match _is_fscomp_present(potname) {
        Ok(x) => x,
        Err(_e) => false,
    }
}

fn _is_pot_present(potname: &str) -> Result<bool, Error> {
    let output = Command::new("pot").args(&["ls", "-q"]).output()?;
    if output.status.success() {
        let output_str = String::from_utf8(output.stdout)?;
        if output_str.lines().any(|x| x == potname) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn is_pot_present(potname: &str) -> bool {
    match _is_pot_present(potname) {
        Ok(x) => x,
        Err(_e) => false,
    }
}

pub(crate) fn destroy_fscomp(fscompname: &str) -> Result<(), Error> {
    let output = Command::new("pot")
        .args(&["destroy", "-f", &fscompname])
        .output()?;
    if !output.status.success() {
        Err(Error::from(PotError::FscompDestroyFailed {
            name: fscompname.to_string(),
        }))
    } else {
        Ok(())
    }
}

fn _destroy_pot(pot_name: &str) -> Result<(), Error> {
    let output = Command::new("pot")
        .args(&["destroy", "-p", &pot_name])
        .output()?;
    if !output.status.success() {
        Err(Error::from(PotError::PotDestroyFailed {
            name: pot_name.to_string(),
        }))
    } else {
        Ok(())
    }
}

pub(crate) fn destroy_pot(pot_name: &str) -> Result<(), Error> {
    let output = Command::new("pot").args(&["stop", &pot_name]).output()?;
    if !output.status.success() {
        Err(Error::from(PotError::PotStopFailed {
            name: pot_name.to_string(),
        }))
    } else {
        let ten_minutes = time::Duration::from_millis(10 * 60 * 1000);
        let ten_seconds = time::Duration::from_millis(10 * 1000);
        let start_timestamp = time::Instant::now();
        loop {
            if _destroy_pot(pot_name).is_ok() {
                println!("Really destroyed pot {}", pot_name);
                break;
            }
            if start_timestamp.elapsed() < ten_minutes {
                thread::sleep(ten_seconds);
            } else {
                return Err(Error::from(PotError::PotDestroyFailed {
                    name: pot_name.to_string(),
                }));
            }
        }
        Ok(())
    }
}

pub(crate) fn revert_fscomp(fscomp_name: &str) -> Result<(), Error> {
    let output = Command::new("pot")
        .args(&["revert", "-f", &fscomp_name])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::FscompRevertFailed {
            name: fscomp_name.to_string(),
            snap: "source_only".to_string(),
        }));
    }
    Ok(())
}

pub(crate) fn get_fscomp_path(fscomp_name: &str) -> Result<String, Error> {
    let output = Command::new("pot")
        .args(&["config", "-qg", "fscomp_prefix"])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::FscompPrefix));
    }
    let prefix = String::from_utf8(output.stdout)?;
    let prefix = prefix.trim_end();
    let fscomp_path = format!("{}/{}", prefix, fscomp_name);
    Ok(fscomp_path)
}

pub(crate) fn get_pot_path(pot_name: &str) -> Result<String, Error> {
    let output = Command::new("pot")
        .args(&["config", "-qg", "pot_prefix"])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::PotPrefix));
    }
    let prefix = String::from_utf8(output.stdout)?;
    let prefix = prefix.trim_end();
    let pot_path = format!("{}/{}", prefix, pot_name);
    Ok(pot_path)
}

pub(crate) fn fetch_git_in_fscomp(
    repo: &Project,
    repo_status: &RepoStatus,
    config: &Opt,
) -> Result<String, Error> {
    let fscomp_name = repo.to_string();
    let fscomp_path = get_fscomp_path(&fscomp_name)?;
    if is_fscomp_present(&fscomp_name) {
        println!("fscomp {} found", fscomp_name);
        if config.force_flag {
            /* Delete the fscomp */
            destroy_fscomp(&fscomp_name)?;
            println!("fscomp {} destroyed", fscomp_name);
        } else {
            return Err(Error::from(PotError::FscompAlreadyPresent {
                name: fscomp_name,
            }));
        }
    } else {
        println!("no fscomp {} found", fscomp_name);
    }
    /* create the fscomp */
    let output = Command::new("pot")
        .args(&["create-fscomp", "-f", &fscomp_name])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::FscompCreateFailed {
            name: fscomp_name,
        }));
    }
    /* git clone in it */
    if let Some(tag) = &config.tag_name {
        let output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "--branch",
                tag,
                repo_status.url.as_str(),
                &fscomp_path,
            ])
            .output()?;
        if !output.status.success() {
            return Err(Error::from(PotError::GitCloneTagFailed {
                url: repo_status.url.as_str().to_string(),
                path: fscomp_path,
                tag: tag.to_string(),
                stderr: String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "stderr not available".to_string()),
            }));
        }
    } else {
        let output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                repo_status.url.as_str(),
                &fscomp_path,
            ])
            .output()?;
        if !output.status.success() {
            return Err(Error::from(PotError::GitCloneFailed {
                url: repo_status.url.as_str().to_string(),
                path: fscomp_path,
                stderr: String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "stderr not available".to_string()),
            }));
        }
    }
    /* take a snapshot */
    let output = Command::new("pot")
        .args(&["snapshot", "-f", &fscomp_name])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::FscompSnapshotFailed {
            name: fscomp_name,
            snap: "source_only".to_string(),
        }));
    }
    Ok(fscomp_path)
}

pub(crate) fn spawn_builder_pot(
    parent_pot: &str,
    fscomp_name: &str,
    config: &Opt,
) -> Result<String, Error> {
    let mut pot_name = parent_pot.to_string();
    pot_name.push('-');
    pot_name.push_str(fscomp_name);
    if is_pot_present(&pot_name) {
        if config.force_flag {
            _destroy_pot(&pot_name)?;
        } else {
            return Err(Error::from(PotError::PotAlreadyPresent { name: pot_name }));
        }
    }
    let output = Command::new("pot")
        .args(&["clone", "-f", "-P", &parent_pot, "-p", &pot_name])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::PotCloneFailed {
            name: pot_name,
            parent: parent_pot.to_string(),
        }));
    }
    // attach the fscomp to the cloned pot
    let output = Command::new("pot")
        .args(&[
            "add-fscomp",
            "-p",
            &pot_name,
            "-f",
            &fscomp_name,
            "-m",
            "/mnt",
        ])
        .output()?;
    if !output.status.success() {
        return Err(Error::from(PotError::AddFscompFailed {
            pot: pot_name,
            fscomp: fscomp_name.to_string(),
            mnt: "/mnt".to_string(),
        }));
    }

    Ok(pot_name)
}
