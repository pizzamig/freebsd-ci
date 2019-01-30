mod builder;
mod config;
mod error;
mod github;
mod pot;
mod yaml;
use crate::builder::build;
use crate::error::ParseError;
use crate::github::{get_release_id, get_status, AssetJson};
use crate::yaml::{
    get_build_lang, get_build_os, get_lang, get_no_deploy, get_os, get_update, get_yaml,
};
use exitfailure::ExitFailure;
use failure::ResultExt;
use log::{debug, error, info};
use std::fmt::Display;
use std::path::PathBuf;
use std::string::ToString;
use structopt::StructOpt;
use yaml_rust::YamlLoader;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    verbose: structopt_flags::SimpleVerbose,
    /// The pathname to the toml configuration file
    #[structopt(
        short = "-c",
        long = "--config",
        parse(from_os_str),
        default_value = "./freebsd-ci.conf"
    )]
    configfile: PathBuf,
    /// The pathname to the build-sh template
    #[structopt(
        short = "-b",
        long = "--build",
        parse(from_os_str),
        default_value = "./templates/build.sh"
    )]
    build_template: PathBuf,
    /// A Flag to force operations (i.e. remove fscomp or images with the same name)
    #[structopt(short = "-f", long = "--force")]
    force_flag: bool,
    /// A Flag to rendert the build script only (on stdout)
    #[structopt(short = "-B", long = "--build-script-only")]
    render_build_flag: bool,
    /// Github project name
    #[structopt(short = "-P", long = "--project")]
    project_name: String,
    /// Github user name
    #[structopt(short = "-U", long = "--user-name")]
    user_name: String,
    /// Tag name: Using this option, a tag will be built. If a related release is found,
    /// the artifacts will be uploaded
    #[structopt(short = "-T", long = "--tag-name")]
    tag_name: Option<String>,
}

#[derive(Debug, Clone)]
struct BuildLang {
    lang: String,
    lang_variant: String,
}

impl Display for BuildLang {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "BuildLang {} (variant: {})",
            self.lang, self.lang_variant
        )
    }
}

#[derive(Debug, Clone)]
struct BuildOS {
    os_family: String,
    os_version: String,
}

impl Display for BuildOS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "BuildOS {} {}", self.os_family, self.os_version)
    }
}

#[derive(Debug)]
struct BuildJob {
    lang: BuildLang,
    os: BuildOS,
    deploy: bool,
}

fn print_jobs(jv: &[BuildJob]) {
    println!("\nJob queue of {} elements", jv.len());
    for j in jv {
        println!(
            "BuildJob: {} {}  {} ({}) deploy: {}",
            j.os.os_family, j.os.os_version, j.lang.lang, j.lang.lang_variant, j.deploy
        );
    }
}

impl ToString for BuildJob {
    fn to_string(&self) -> String {
        let mut rc = self.os.os_family.clone();
        rc.push('-');
        rc.push_str(&self.os.os_version);
        rc.push('-');
        rc.push_str(&self.lang.lang);
        rc.push('-');
        rc.push_str(&self.lang.lang_variant);
        rc.replace(".", "_")
    }
}

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) owner: String,
    pub(crate) project: String,
}

impl ToString for Project {
    fn to_string(&self) -> String {
        format!("{}__{}", self.owner, self.project)
    }
}

#[derive(Debug, Default)]
pub(crate) struct BuildOpt {
    pub(crate) update: bool,
    pub(crate) release_id: Option<u64>,
    pub(crate) assets: Vec<AssetJson>,
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    env_logger::try_init()?;
    debug!("BSD Continuous integration");
    /* Initial checks */
    if !crate::pot::is_pot_available() {
        error!("This error needs better explanation");
        return Ok(());
    }
    debug!(
        "Reading configuration file {}",
        opt.configfile
            .to_str()
            .unwrap_or("Filename not convertible")
    );
    let config = crate::config::get_config(&opt.configfile).with_context(|_| {
        format!(
            "could not parse the file {}",
            opt.configfile.to_str().unwrap_or("file name not printable")
        )
    })?;
    let prj = Project {
        owner: opt.user_name.clone(),
        project: opt.project_name.clone(),
    };
    let (rs, _) = get_status(&prj, &config.tokens.github).with_context(|_| {
        format!(
            "Fecth repository data failed for user {} project {}",
            prj.owner, prj.project,
        )
    })?;
    println!("github repository information:\n{}", rs);
    /* fetch the repo to read the .bsd-ci file */
    let path = crate::pot::fetch_git_in_fscomp(&prj, &rs, &opt).with_context(|_| {
        "Failed to create a ZFS dataset with the project in it\n Is pot installed?\n Are you root?"
    })?;

    println!("Git repo fetched in {}", path);

    let mut build_queue = Vec::new();
    let mut build_opt = BuildOpt::default();
    let yaml_string =
        get_yaml(&path).with_context(|_| "Error accessing the yaml file in the project")?;
    let docs = YamlLoader::load_from_str(&yaml_string)
        .with_context(|_| "Error parsing the yaml file in the project")?;
    for d in docs {
        let h = d.into_hash().unwrap();
        debug!("{:?}", h);
        let lang = get_lang(&h).with_context(|_| "Invalid YAML")?;
        let os = get_os(&h).with_context(|_| "Invalid YAML")?;
        let build_lang = match lang.as_ref() {
            "rust" => get_build_lang("rust", &h)
                .with_context(|_| format!("Ivalid YAML for language {}", lang))?,
            _ => {
                return Err(ExitFailure::from(ParseError::GenericError {
                    msg: "language not supported".to_string(),
                }));
            }
        };
        let build_os = match os.as_ref() {
            "FreeBSD" => get_build_os("FreeBSD", &h)?,
            _ => {
                return Err(ExitFailure::from(ParseError::GenericError {
                    msg: "os not supported".to_string(),
                }));
            }
        };
        build_opt.update = get_update(&h)?;
        info!("{:?}", build_lang);
        info!("{:?}", build_os);
        for o in &build_os {
            for l in &build_lang {
                build_queue.push(BuildJob {
                    lang: l.clone(),
                    os: o.clone(),
                    deploy: true,
                });
                debug!("o {:?} - l {:?}", o, l);
            }
        }
        get_no_deploy(&h, &mut build_queue)?;
        print_jobs(&build_queue);
    }
    if let Some(tag_name) = &opt.tag_name {
        if let Ok((release_id, assets, _)) = get_release_id(&prj, tag_name, &config.tokens.github) {
            build_opt.release_id = Some(release_id);
            build_opt.assets = assets;
        }
    }
    build(&build_queue, &prj, &opt, &build_opt, &config.tokens.github)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_to_string_rust() {
        let uut = BuildJob {
            os: BuildOS {
                os_family: "FreeBSD".to_string(),
                os_version: "11.2".to_string(),
            },
            lang: BuildLang {
                lang: "rust".to_string(),
                lang_variant: "stable".to_string(),
            },
            deploy: true,
        };
        let rc = uut.to_string();
        assert_eq!(&rc, "FreeBSD-11_2-rust-stable");
    }
    #[test]
    fn test_job_to_string_php() {
        let uut = BuildJob {
            os: BuildOS {
                os_family: "FreeBSD".to_string(),
                os_version: "12.0".to_string(),
            },
            lang: BuildLang {
                lang: "php".to_string(),
                lang_variant: "7.3".to_string(),
            },
            deploy: false,
        };
        let rc = uut.to_string();
        assert_eq!(&rc, "FreeBSD-12_0-php-7_3");
    }

    #[test]
    fn test_project_to_string_ok() {
        let uut = Project {
            owner: "pizzamig".to_string(),
            project: "potnet".to_string(),
        };
        let rc = uut.to_string();
        assert_eq!(&rc, "pizzamig__potnet");
    }
}
