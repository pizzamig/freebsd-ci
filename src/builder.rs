use crate::pot::{
    destroy_fscomp, destroy_pot, get_pot_path, is_pot_present, revert_fscomp, spawn_builder_pot,
    PotError,
};
use crate::{BuildJob, BuildOpt, Opt, Project};
use failure::{Error, Fail};
use log::debug;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use tera::{Context, Tera};

#[derive(Debug, Fail)]
pub(crate) enum BuildError {
    #[fail(display = "Missing pot: {}", potname)]
    PotNotPresent { potname: String },
    #[fail(display = "Tera template parsing error: {}", msg)]
    TeraTemplateParseErr { msg: String },
    #[fail(display = "Tera template rendering error: {}", msg)]
    TeraTemplateRenderingErr { msg: String },
}

fn generate_build_script(
    pot_name: &str,
    job: &BuildJob,
    prj: &Project,
    build_opt: &BuildOpt,
    token: &str,
    opt: &Opt,
) -> Result<(), Error> {
    let mut template_dir = opt
        .build_template
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    println!("template_dir 1 : {}", template_dir);
    if &template_dir == "" {
        template_dir.push_str(".");
    }
    println!("template_dir 1 : {}", template_dir);
    template_dir.push_str("/*");
    println!("template_dir 3 : {}", template_dir);

    // render the template
    let tera = match Tera::new(&template_dir) {
        Ok(t) => t,
        Err(e) => {
            return Err(Error::from(BuildError::TeraTemplateParseErr {
                msg: format!("{}", e),
            }));
        }
    };
    let mut context = Context::new();
    context.insert("update", &build_opt.update);
    context.insert("language", &job.lang.lang);
    context.insert("language_variant", &job.lang.lang_variant);
    context.insert("os_family", &job.os.os_family);
    context.insert("os_version", &job.os.os_version);
    context.insert("user", &prj.owner);
    context.insert("project", &prj.project);
    if let Some(release_id) = build_opt.release_id {
        context.insert("upload", &true);
        context.insert("token", token);
        context.insert("release_id", &release_id);
    } else {
        context.insert("upload", &false);
        context.insert("token", "");
        context.insert("release_id", &0);
    }
    let script = match tera.render("build.sh", &context) {
        Ok(s) => s,
        Err(e) => {
            return Err(Error::from(BuildError::TeraTemplateRenderingErr {
                msg: format!("{}", e),
            }));
        }
    };
    if opt.render_build_flag {
        println!("{}", script);
    } else {
        // write the script to a file
        let pot_path = get_pot_path(pot_name)?;
        let mut file_path = PathBuf::from(&pot_path);
        file_path.push("m");
        file_path.push("root");
        file_path.push("build.sh");
        debug!("Creating the file {:?}", file_path);
        let mut f = File::create(&file_path)?;
        let metadata = f.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        f.set_permissions(permissions)?;
        write!(f, "{}", script)?;
    }
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

pub(crate) fn build(
    queue: &[BuildJob],
    prj: &Project,
    opt: &Opt,
    build_opt: &BuildOpt,
    token: &str,
) -> Result<(), Error> {
    let fscomp_name = prj.to_string();
    for b in queue {
        let image_name = b.to_string();
        if !is_pot_present(&image_name) {
            return Err(Error::from(BuildError::PotNotPresent {
                potname: image_name,
            }));
        }

        // spawn the container
        let pot_name = spawn_builder_pot(&image_name, &fscomp_name, &opt)?;
        println!("Spawned new pot: {}", pot_name);
        // run the build
        generate_build_script(&pot_name, b, prj, build_opt, &token, opt)?;
        if opt.render_build_flag {
            destroy_pot(&pot_name)?;
            destroy_fscomp(&fscomp_name)?;
            return Ok(());
        }
        run_build_script(&pot_name)?;
        // cleanup
        // // destroy the pot
        destroy_pot(&pot_name)?;
        debug!("Destroyed pot: {}", pot_name);
        // // revert the fscomp
        revert_fscomp(&fscomp_name)?;
        debug!("Revert fscomp : {}", fscomp_name);
    }
    // destroy the fscomp
    destroy_fscomp(&fscomp_name)?;
    Ok(())
}
