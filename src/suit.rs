use std::path::PathBuf;
use std::path::Path;
use crate::BoxErr;
use crate::format;
use format::suit::TestSuitCfg;

pub struct TestSuite<Cfg> {
    cfg: Cfg,
}

impl<Cfg> TestSuite<Cfg> {
    pub fn new(cfg: Cfg) -> Self {
        //
        Self { cfg }
    }
}

pub fn load_test_suit(path: &Path) -> Result<TestSuitCfg, BoxErr> {
    println!("loading test-suit {}", path.display());
    let f = std::fs::File::open(path)?;
    let res = format::suit::deserialize(f).map(|mut cfg: TestSuitCfg| {
                                              // add fallback values if needed
                                              if cfg.name.is_none() {
                                                  if let Some(name) = path.file_name()
                                                                          .map(|f| f.to_str())
                                                                          .flatten()
                                                                          .map(|s| s.to_string())
                                                  {
                                                      cfg.name = Some(name);
                                                  }
                                              }
                                              cfg
                                          });
    println!("deserialize test-suit {:?}", res);
    Ok(res?)
}

pub fn load_requested_suits(path: &Path) -> Result<impl Iterator<Item = TestSuitCfg>, BoxErr> {
    Ok(get_suits_files(path)?.into_iter()
                             .filter_map(|f| load_test_suit(&f).ok()))
}

pub fn get_suits_files(path: &Path) -> Result<Vec<PathBuf>, BoxErr> {
    let suits = if path.is_dir() {
        std::fs::read_dir(path)?.filter_map(|f| {
                                    let path = f.ok().map(|f| f.path());

                                    if let Some(p) = path {
                                        println!("path: {}", p.as_path().display());
                                        if let Some(ext) = p.extension()
                                                            .map(|f| f.to_str())
                                                            .flatten()
                                                            .map(|s| s.to_lowercase())
                                        {
                                            if ext == "yml" || ext == "yaml" {
                                                Some(p)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
    } else if path.exists() && path.is_file() {
        vec![path.to_owned()]
    } else {
        vec![]
    };

    Ok(suits)
}
