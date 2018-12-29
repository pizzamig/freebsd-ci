mod builder;
mod config;
mod error;
mod github;
mod pot;
mod yaml;
use crate::builder::build;
use crate::config::Project;
use crate::error::ParseError;
use crate::github::get_status;
use crate::yaml::{get_build_lang, get_build_os, get_lang, get_os, get_yaml};
use failure::Error;
use log::{debug, error, info};
use std::path::PathBuf;
use std::string::ToString;
use structopt::StructOpt;
use yaml_rust::YamlLoader;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    verbose: structopt_flags::SimpleVerbose,
    /// The pathname to the toml configuration file
    #[structopt(short = "-c", parse(from_os_str), default_value = "./bsd-ci.conf")]
    configfile: PathBuf,
    /// A Flag to force operations
    #[structopt(short = "-f")]
    force_flag: bool,
}

#[derive(Debug, Clone)]
struct BuildLang {
    lang: String,
    lang_variant: String,
}

#[derive(Debug, Clone)]
struct BuildOS {
    os_family: String,
    os_version: String,
}

#[derive(Debug)]
struct BuildJob {
    lang: BuildLang,
    os: BuildOS,
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

fn main() -> Result<(), Error> {
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
    let config = crate::config::get_config(&opt.configfile)?;
    let prj = config.projects.first().unwrap();
    let (rs, _) = get_status(&prj, &config.tokens.github)?;
    println!("{:?}", rs);
    /* fetch the repo to read the .bsd-ci file */
    let path = crate::pot::fetch_git_in_fscomp(&prj, &rs, &opt)?;

    println!("Git repo fetched in {}", path);

    let mut build_queue = Vec::new();
    let yaml_string = get_yaml(&path)?;
    let docs = YamlLoader::load_from_str(&yaml_string)?;
    for d in docs {
        let h = d.into_hash().unwrap();
        debug!("{:?}", h);
        let lang = get_lang(&h)?;
        let os = get_os(&h)?;
        let build_lang = match lang.as_ref() {
            "rust" => get_build_lang("rust", &h)?,
            _ => {
                return Err(Error::from(ParseError::GenericError {
                    msg: "language not supported".to_string(),
                }))
            }
        };
        let build_os = match os.as_ref() {
            "FreeBSD" => get_build_os("FreeBSD", &h)?,
            _ => {
                return Err(Error::from(ParseError::GenericError {
                    msg: "os not supported".to_string(),
                }))
            }
        };
        for o in &build_os {
            for l in &build_lang {
                build_queue.push(BuildJob {
                    lang: l.clone(),
                    os: o.clone(),
                });
                debug!("o {:?} - l {:?}", o, l);
            }
        }
        info!("{:?}", build_lang);
        info!("{:?}", build_os);
        println!("{:?}", build_queue);
    }
    build(&build_queue, prj, &opt)?;
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
