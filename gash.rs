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
use std::io::signal::Listener;
use std::io::signal::Interrupt;

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

            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                "cd"    =>  { self.cd(cmd_line); }
                "history"    =>  { self.history(cmd_history.clone()); }
                _ => {}
            }

            match args.clone().last() {
                Some("&") => {
                    spawn(proc() {
                        let mut listener = Listener::new();
                        listener.register(Interrupt);
                        loop {
                            match listener.port.recv() {
                                Interrupt => (),
                                _ => (),
                            }
                        }
                    });
                    println("found & symbol");
                    let mut iter = args.clone();
                    let mut past = ~[];
                    let mut prog = program;
                    loop {
                        let token = iter.next();
                        let x = token.unwrap();
                        if x == "&" {
                            // let prog2 = prog;
                            // let past2 = past;
                            // spawn(proc() { run::process_status(prog2, past2) });
                            break;
                        }
                        match x {
                            "<" => {
                                let mut arg_list: ~[~str] = past.clone();
                                arg_list.push(iter.next().unwrap().to_owned());
                                let buffer = run::process_output(prog, arg_list);
                                past = ~[];
                                past.push(std::str::from_utf8_owned(buffer.unwrap().output));
                                continue;
                            }
                            ">" => {
                                let arg_list: ~[~str] = past.clone();
                                let buffer = run::process_output(prog, arg_list);
                                let path = &Path::new(iter.next().unwrap());
                                let output = File::open_mode(path, io::Truncate, io::Write);
                                match output {
                                    Some(mut file) => file.write(buffer.unwrap().output),
                                    None => fail!("Error: Could not write to file.")
                                }
                                continue;
                            }
                            "|" => {
                                println("found | symbol");
                                let arg_list: ~[~str] = past.clone();
                                let buffer = run::process_output(prog, arg_list);
                                past = ~[];
                                past.push(std::str::from_utf8_owned(buffer.unwrap().output));
                                prog = iter.next().unwrap();
                                continue;
                            }
                            _ => past.push(x.to_owned())
                        }
                    }
                    continue;
                }
                _ => {
                    let mut iter = args.clone();
                    let mut past = ~[];
                    let mut prog = program;
                    loop {
                        let token = iter.next();
                        if token == None {
                            run::process_status(prog, past);
                            break;
                        }
                        let x = token.unwrap();
                        match x {
                            "<" => {
                                let mut arg_list: ~[~str] = past.clone();
                                let y = iter.next().unwrap();
                                print("filename: ");
                                println(y);
                                arg_list.push(y.to_owned());
                                // let buffer = run::process_output(prog, arg_list);
                                // past = ~[];
                                // past.push(std::str::from_utf8_owned(buffer.unwrap().output));
                                continue;
                            }
                            ">" => {
                                let arg_list: ~[~str] = past.clone();
                                let buffer = run::process_output(prog, arg_list);
                                let path = &Path::new(iter.next().unwrap());
                                let output = File::open_mode(path, io::Truncate, io::Write);
                                match output {
                                    Some(mut file) => file.write(buffer.unwrap().output),
                                    None => fail!("Error: Could not write to file.")
                                }
                                continue;
                            }
                            "|" => {
                                let arg_list: ~[~str] = past.clone();
                                let buffer = run::process_output(prog, arg_list);
                                past = ~[];
                                let path = &Path::new("temp.txt");
                                File::create(path).write(buffer.unwrap().output);
                                past.push(~"temp.txt");
                                prog = iter.next().unwrap();
                                continue;
                            }
                            _ => past.push(x.to_owned())
                        }
                    }
                    continue;
                }
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
            println("running command line");
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