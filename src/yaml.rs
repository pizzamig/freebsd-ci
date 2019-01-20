use super::{BuildLang, BuildOS};
use crate::error::ParseError;
use log::debug;
use log::error;
use log::info;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub(crate) fn get_yaml(repopath: &str) -> Result<String, std::io::Error> {
    let mut yaml_file = PathBuf::new();
    yaml_file.push(repopath);
    yaml_file.push(".bsd-ci.yml");
    let file = File::open(&yaml_file)?;
    let mut reader = std::io::BufReader::new(file);
    let mut rc = String::new();
    reader.read_to_string(&mut rc)?;
    Ok(rc)
}

pub(crate) fn get_lang(h: &yaml_rust::yaml::Hash) -> Result<String, ParseError> {
    if let Some(k_lang) = h.get(&yaml_rust::Yaml::from_str("language")) {
        if let Some(lang) = k_lang.as_str() {
            debug!("{:?}", lang);
            return Ok(lang.to_string());
        } else {
            error!("language is not a string");
            return Err(ParseError::InvalidType {
                name: "language".to_string(),
            });
        }
    } else {
        error!("No language found!");
        return Err(ParseError::MissingKey {
            key: "language".to_string(),
        });
    }
}

pub(crate) fn get_os(h: &yaml_rust::yaml::Hash) -> Result<String, ParseError> {
    if let Some(k_os) = h.get(&yaml_rust::Yaml::from_str("os")) {
        if let Some(os) = k_os.as_str() {
            debug!("{:?}", os);
            return Ok(os.to_string());
        } else {
            error!("os is not a string");
            return Err(ParseError::InvalidType {
                name: "os".to_string(),
            });
        }
    } else {
        error!("No os found!");
        return Err(ParseError::MissingKey {
            key: "os".to_string(),
        });
    }
}

pub(crate) fn get_build_lang(
    l: &str,
    h: &yaml_rust::yaml::Hash,
) -> Result<Vec<BuildLang>, ParseError> {
    let mut result = Vec::new();
    if let Some(lang) = h.get(&yaml_rust::Yaml::from_str(l)) {
        if let Some(v_lang) = lang.as_vec() {
            for v in v_lang {
                if let Some(v_str) = v.as_str() {
                    result.push(BuildLang {
                        lang: l.to_string(),
                        lang_variant: v_str.to_string(),
                    });
                } else {
                    return Err(ParseError::GenericError {
                        msg: "Language array has invalid type".to_string(),
                    });
                }
            }
        } else {
            return Err(ParseError::InvalidType {
                name: l.to_string(),
            });
        }
    } else {
        error!("No language {} found!", l);
        return Err(ParseError::MissingKey { key: l.to_string() });
    }
    Ok(result)
}

pub(crate) fn get_build_os(
    os: &str,
    h: &yaml_rust::yaml::Hash,
) -> Result<Vec<BuildOS>, ParseError> {
    let mut result = Vec::new();
    if let Some(k_os) = h.get(&yaml_rust::Yaml::from_str(os)) {
        if let Some(v_os) = k_os.as_vec() {
            for v in v_os {
                if let Some(v_str) = v.as_str() {
                    result.push(BuildOS {
                        os_family: os.to_string(),
                        os_version: v_str.to_string(),
                    });
                } else {
                    return Err(ParseError::GenericError {
                        msg: "OS array has invalid type".to_string(),
                    });
                }
            }
        } else {
            return Err(ParseError::InvalidType {
                name: os.to_string(),
            });
        }
    } else {
        error!("No os {} found!", os);
        return Err(ParseError::MissingKey {
            key: os.to_string(),
        });
    }
    Ok(result)
}

pub(crate) fn get_update(h: &yaml_rust::yaml::Hash) -> Result<bool, ParseError> {
    if let Some(k_update) = h.get(&yaml_rust::Yaml::from_str("update")) {
        if let Some(update) = k_update.as_bool() {
            debug!("{:?}", update);
            return Ok(update);
        } else {
            error!("update is not a boolean value");
            return Err(ParseError::InvalidType {
                name: "update".to_string(),
            });
        }
    } else {
        info!("update not found: default to false");
        return Ok(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_lang_ok() {
        let docs = yaml_rust::YamlLoader::load_from_str("language: rust").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let lang = get_lang(&h).unwrap();
        assert_eq!("rust", &lang);
    }
    #[test]
    fn test_get_lang_norust() {
        let docs = yaml_rust::YamlLoader::load_from_str("language: ruby").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let lang = get_lang(&h).unwrap();
        assert_ne!("rust", &lang);
    }
    #[test]
    fn test_get_lang_nolang() {
        let docs = yaml_rust::YamlLoader::load_from_str("lang: rust").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let lang_err: ParseError = get_lang(&h).unwrap_err();
        assert!(lang_err.is_missingkey());
    }
    #[test]
    fn test_get_lang_wrongtype() {
        let docs = yaml_rust::YamlLoader::load_from_str("language: 123").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let lang_err: ParseError = get_lang(&h).unwrap_err();
        assert!(lang_err.is_invalidtype());
    }
    #[test]
    fn test_get_lang_array() {
        let docs = yaml_rust::YamlLoader::load_from_str("language:\n  - rust\n  - ruby").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let lang_err: ParseError = get_lang(&h).unwrap_err();
        assert!(lang_err.is_invalidtype());
    }

    #[test]
    fn test_get_os_ok() {
        let docs = yaml_rust::YamlLoader::load_from_str("os: FreeBSD").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let os = get_os(&h).unwrap();
        assert_eq!("FreeBSD", &os);
    }
    #[test]
    fn test_get_os_norust() {
        let docs = yaml_rust::YamlLoader::load_from_str("os: Linux").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let os = get_os(&h).unwrap();
        assert_ne!("FreeBSD", &os);
    }
    #[test]
    fn test_get_os_nolang() {
        let docs = yaml_rust::YamlLoader::load_from_str("os_family: FreeBSD").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let os_err: ParseError = get_os(&h).unwrap_err();
        assert!(os_err.is_missingkey());
    }
    #[test]
    fn test_get_os_wrongtype() {
        let docs = yaml_rust::YamlLoader::load_from_str("os: 11.2").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let os_err: ParseError = get_os(&h).unwrap_err();
        assert!(os_err.is_invalidtype());
    }
    #[test]
    fn test_get_os_array() {
        let docs = yaml_rust::YamlLoader::load_from_str("os:\n  - FreeBSD\n  - osx").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let os_err: ParseError = get_os(&h).unwrap_err();
        assert!(os_err.is_invalidtype());
    }

    #[test]
    fn test_get_build_lang_ok() {
        let docs = yaml_rust::YamlLoader::load_from_str("rust:\n  - nightly\n  - stable").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let mut bl = get_build_lang("rust", &h).unwrap().into_iter();
        assert!(bl
            .find(|x| x.lang == "rust" && x.lang_variant == "nightly")
            .is_some());
        assert!(bl
            .find(|x| x.lang == "rust" && x.lang_variant == "stable")
            .is_some());
        assert!(bl
            .find(|x| x.lang == "rust" && x.lang_variant == "beta")
            .is_none());
    }
    #[test]
    fn test_get_build_lang_noname() {
        let docs = yaml_rust::YamlLoader::load_from_str("rust:\n  - 1.32\n  - 1.30").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bl_err: ParseError = get_build_lang("rust", &h).unwrap_err();
        assert!(bl_err.is_genericerror());
    }
    #[test]
    fn test_get_build_lang_noarray() {
        let docs = yaml_rust::YamlLoader::load_from_str("rust: stable\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bl_err: ParseError = get_build_lang("rust", &h).unwrap_err();
        assert!(bl_err.is_invalidtype());
    }
    #[test]
    fn test_get_build_lang_norust() {
        let docs = yaml_rust::YamlLoader::load_from_str("ruby:\n  .  stable\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bl_err: ParseError = get_build_lang("rust", &h).unwrap_err();
        assert!(bl_err.is_missingkey());
    }

    #[test]
    fn test_get_build_os_ok() {
        let docs =
            yaml_rust::YamlLoader::load_from_str("FreeBSD:\n  - '11.2'\n  - '12.0'\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let mut bo = get_build_os("FreeBSD", &h).unwrap().into_iter();
        assert!(bo
            .find(|x| x.os_family == "FreeBSD" && x.os_version == "11.2")
            .is_some());
        assert!(bo
            .find(|x| x.os_family == "FreeBSD" && x.os_version == "12.0")
            .is_some());
        assert!(bo
            .find(|x| x.os_family == "FreeBSD" && x.os_version == "10.4")
            .is_none());
    }
    #[test]
    fn test_get_build_os_nostr() {
        let docs = yaml_rust::YamlLoader::load_from_str("FreeBSD:\n  - 11.2\n  - 12.0\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bo_err = get_build_os("FreeBSD", &h).unwrap_err();
        assert!(bo_err.is_genericerror());
    }
    #[test]
    fn test_get_build_os_noarray() {
        let docs = yaml_rust::YamlLoader::load_from_str("FreeBSD: '11.2'\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bo_err = get_build_os("FreeBSD", &h).unwrap_err();
        assert!(bo_err.is_invalidtype());
    }
    #[test]
    fn test_get_build_os_noos() {
        let docs = yaml_rust::YamlLoader::load_from_str("Darwin:\n  - '11.2'\n").unwrap();
        let d = docs.first().unwrap();
        let h = d.as_hash().unwrap();
        let bo_err = get_build_os("FreeBSD", &h).unwrap_err();
        assert!(bo_err.is_missingkey());
    }
}
