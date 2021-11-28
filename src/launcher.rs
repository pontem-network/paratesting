use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use async_trait::async_trait;

extern crate subprocess;
use subprocess::Exec;
use subprocess::Redirection;

use crate::BoxErr;
use crate::format;
use format::suit::ConditionsCfg;
use format::suit::PolkaLaunchCfg;
use format::suit::ProcessRunCfg;

#[async_trait]
pub trait Launcher {
    fn conditions(&self) -> &ConditionsCfg;
    fn pwd(&self) -> Option<&Path>;
    fn cmd(&self) -> &str;

    async fn exec(&mut self) -> Result<(), BoxErr> {
        //
        Ok(())
    }

    fn run(&mut self) {
        // build subprocess:
        // TODO: support stderr reading without merging into stdout
        let mut exec = Exec::shell(self.cmd()).stdout(Redirection::Pipe)
                                              .stderr(Redirection::Pipe);
        if let Some(pwd) = self.pwd() {
            exec = exec.cwd(pwd);
        }

        // TODO: like bottom
        // let mut p = exec.popen().unwrap();

        // let (mut stdout, mut stderr) = (&mut p.stdout, &mut p.stderr);
        // let (mut stdout, mut stderr) = (p.stdout.as_mut(), p.stderr.as_mut());
        // let mut out = stdout.map(|s| BufReader::new(s));
        // let err = stderr.map(|s| BufReader::new(s));

        if let Some(conditions) = self.conditions().success.as_ref() {
            use format::suit::Conditions;

            // let (buf, expected): (BufReader<&mut dyn std::io::Read>, _) = match conditions {
            let (boxed, expected, limit): (Box<dyn std::io::Read>, _, _) = match conditions {
                Conditions::Stdout(s) => {
                    // TODO: spawn watcher for:
                    (Box::new(exec.stream_stdout().unwrap()), s, None)
                }
                Conditions::Stderr(s) => {
                    // TODO: spawn watcher for:
                    (Box::new(exec.stream_stderr().unwrap()), s, None)
                }

                Conditions::Wait { box conditions,
                                   limit, } => {
                    match conditions {
                        Conditions::Stdout(s) => {
                            // TODO: spawn watcher for:
                            (Box::new(exec.stream_stdout().unwrap()), s, Some(limit))
                        }
                        Conditions::Stderr(s) => {
                            // TODO: spawn watcher for:
                            (Box::new(exec.stream_stderr().unwrap()), s, Some(limit))
                        }
                        _ => unimplemented!("not supported"),
                    }
                }

                _ => unimplemented!("not supported"),
            };

            let buf = BufReader::new(boxed);

            use std::time::SystemTime;
            let now = SystemTime::now();
            // blocking checker
            for (i, line) in buf.lines().enumerate() {
                println!("{}: {}", i, line.unwrap());
            }
        }

        // let mut stdout = exec.stream_stdout().unwrap();
        // // let buf: BufReader<&mut dyn std::io::Read> = BufReader::new(&mut stdout);

        // let boxed: Box<dyn std::io::Read> = Box::new(stdout);
        // let buf = BufReader::new(boxed);

        // // blocking checker
        // for (i, line) in buf.lines().enumerate() {
        // 	println!("{}: {}", i, line.unwrap());
        // }
    }
}

pub struct Setup<Cfg> {
    cfg: Cfg,
    conditions: ConditionsCfg,
    proc: Option<()>,
    cmd: String,
}

impl<S: AsRef<str>> Setup<ProcessRunCfg<S>> {
    pub fn new(cfg: ProcessRunCfg<S>, conditions: ConditionsCfg) -> Self {
        let cmd = cfg.cmd.as_ref().to_owned();
        Self { cfg,
               conditions,
               cmd,
               proc: None }
    }
}
impl Setup<PolkaLaunchCfg> {
    pub fn new(cfg: PolkaLaunchCfg, conditions: ConditionsCfg) -> Self {
        let cmd = format!("polkadot-launch {}", &cfg.cfg.display());
        Self { cfg,
               conditions,
               cmd,
               proc: None }
    }
}

impl Launcher for Setup<PolkaLaunchCfg> {
    fn conditions(&self) -> &ConditionsCfg { &self.conditions }
    fn pwd(&self) -> Option<&Path> { self.cfg.inner.pwd.as_ref().map(|p| p.as_path()) }
    fn cmd(&self) -> &str { &self.cmd }

    fn run(&mut self) {
        let mut exec = Exec::shell("tail -f ./test.txt").stdout(Redirection::Pipe)
                                                        .stderr(Redirection::Pipe);
        if let Some(pwd) = self.pwd() {
            exec = exec.cwd(pwd);
        }

        let mut p = exec.popen().unwrap();

        // let (mut stdout, mut stderr) = (&mut p.stdout, &mut p.stderr);
        // let (mut stdout, mut stderr) = (p.stdout.as_mut(), p.stderr.as_mut());
        // let mut out = stdout.map(|s| BufReader::new(s));
        // let err = stderr.map(|s| BufReader::new(s));

        pub struct LinesWatcher<R: BufRead> {
            expected: String,
            // reader: R,
            lines: std::io::Lines<R>,
        }

        impl<R: BufRead> LinesWatcher<R> {
            pub fn watch(&mut self) {
                let res = self.lines
                              .find(|s| {
                                  println!("{:?}", s);
                                  s.as_ref()
                                   .map(|s| s.trim().contains(&self.expected))
                                   .unwrap_or(false)
                              })
                              .is_some();
                println!("END! success: {}", res);
            }
        }

        // let (mut stdout, mut stderr) = (p.stdout.as_mut(), p.stderr.as_mut());

        // if let Some(out) = out.take() {
        let mut a = std::thread::spawn(move || {
            let mut stdout = p.stdout.as_mut();
            let mut out = stdout.map(|s| BufReader::new(s));

            if let Some(out) = out.take() {
                let mut watcher = LinesWatcher { expected: "expected".to_string(),
                                                 lines: out.lines() };
                watcher.watch();
            // return watcher.watch();
            } else {
                // return false;
            }
        });

        println!("main thread is free");
        let res = a.join().unwrap();
        println!("watcher finished with {:?}", res);
        // }
    }
}

impl<S: AsRef<str>> Launcher for Setup<ProcessRunCfg<S>> {
    fn conditions(&self) -> &ConditionsCfg { &self.conditions }
    fn pwd(&self) -> Option<&Path> { self.cfg.pwd.as_ref().map(|p| p.as_path()) }
    fn cmd(&self) -> &str { &self.cmd }
}
