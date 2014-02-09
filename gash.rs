//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::{io, run, os};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use extra::getopts;
use std::io::File;
// use std::run::Process;
// use std::run::ProcessOptions;
// use std::libc;
// extra::future;
// use std::task::task;

struct Shell {
    cmd_prompt: ~str,
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut cmd_history = ~[];

        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            let args = cmd_line.split(' ').skip(1);

            cmd_history.push(cmd_line.clone());

            match args.clone().last() {
                Some("&") => {
                    let arg_list: ~[~str] =
                        args.clone().filter_map(|x| if x != "&" { Some(x.to_owned()) } else { None }).to_owned_vec();
                    let prog = program.to_owned();
                    spawn(proc() { run::process_status(prog, arg_list); });
                    continue;
                }
                _ => {}
            }

            // fix needed: support for command ./zhttpto > zlog.txt &
            let mut iter = args.clone();
            match iter.find(|&x| x==">") {
                Some(">") => {
                    let mut arg_list: ~[~str] = ~[];
                    args.clone().advance(|x| if x != ">" { arg_list.push(x.to_owned()); true } else { false });
                    let buffer = run::process_output(program, arg_list);
                    let path = &Path::new(iter.next().unwrap());
                    let output = File::open_mode(path, io::Truncate, io::Write);
                    match output {
                        Some(mut file) => file.write(buffer.unwrap().output),
                        None => fail!("Error: Could not write to file.")
                    }
                    continue;
                }
                _ => {}
            }

            // note: ambiguity on stdin redirection
            let mut iter = args.clone();
            match iter.find(|&x| x=="<") {
                Some("<") => {
                    let mut arg_list: ~[~str] = ~[];
                    args.clone().advance(|x| if x != "<" { arg_list.push(x.to_owned()); true } else { false });
                    // let file = File::open(&Path::new(iter.next().unwrap()));
                    // match file {
                    //     Some(f) => arg_list.push(f),
                    //     _ => fail!("Error: File not found")
                    // }
                    arg_list.push(iter.next().unwrap().to_owned());
                    run::process_status(program, arg_list);
                    continue;
                }
                _ => {}
            }

            let mut iter = args.clone();
            match iter.find(|&x| x=="|") {
                Some("|") => {
                    let mut arg_list: ~[~str] = ~[];
                    args.clone().advance(|x| if x != "|" { arg_list.push(x.to_owned()); true } else { false });
                    let buffer = run::process_output(program, arg_list);
                    let target = iter.next().unwrap();
                    arg_list = ~[];
                    iter.advance(|x| if x != "" { arg_list.push(x.to_owned()); true } else { false });
                    arg_list.push(std::str::from_utf8_owned(buffer.unwrap().output));
                    run::process_status(target, arg_list);
                    continue;
                }
                _ => {}
            }

            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                "cd"    =>  { self.cd(cmd_line); }
                "history"    =>  { self.history(cmd_history.clone()); }
                _       =>  { self.run_cmdline(cmd_line); }
            }
        }
    }
    
    fn history(&mut self, cmd_history: ~[~str]) {
        for i in range(0, cmd_history.len()) {
            println!("   {}  {:s}", i+1, cmd_history[i]);
        }
    }

    fn cd(&mut self,  cmd_line: &str) {
        let argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 1 {
            let path = &Path::new(argv[1]);
            //println!("path: {}", path.display());
            os::change_dir(path);
        }
    }

    fn run_cmdline(&mut self, cmd_line: &str) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            self.run_cmd(program, argv);
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
            run::process_status(program, argv);
        } else {
            println!("{:s}: command not found", program);
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    
    let opts = ~[
        getopts::optopt("c")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
            Some(cmd_str) => {cmd_str.to_owned()}, 
            None => {~""}
        };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
