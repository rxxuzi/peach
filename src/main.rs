use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "compile" | "c" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                print_usage();
                process::exit(1);
            }

            let input_file = &args[2];
            println!("Compiling: {}", input_file);
            println!();

            match peach::compile(input_file) {
                Ok(_) => {
                    println!();
                    println!("✓ Successfully compiled to C");
                    process::exit(0);
                }
                Err(_) => {
                    // Error already reported by compile function
                    process::exit(1);
                }
            }
        }
        "version" | "-v" | "--version" => {
            println!("peach {}", VERSION);
            process::exit(0);
        }
        "help" | "-h" | "--help" => {
            print_usage();
            process::exit(0);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Usage: peach <command> [options]");
    println!();
    println!("Commands:");
    println!("  compile <file>    Compile Peach file to C");
    println!("  c <file>          Alias for 'compile'");
    println!("  version, -v       Show version");
    println!("  help, -h          Show this help");
}