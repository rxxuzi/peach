use std::process;
use peach::CompileOptions;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let (input_file, options) = match parse_args(&args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Compile with options
    match peach::compile_with_options(&input_file, options) {
        Ok(_) => {
            process::exit(0);
        }
        Err(_) => {
            // Error message already printed by parser or semantic analyzer
            process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> Result<(String, CompileOptions), String> {
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut input_file: Option<String> = None;
    let mut options = CompileOptions {
        verbose: false,        // pc is silent by default
        check_only: false,
        output_path: None,
        warnings: false,
        use_build_dir: true,   // pc uses build directory (tests/ -> tests/out/)
        dry_run: false,
    };

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--version" | "-v" => {
                println!("pc (Peach compiler) {}", VERSION);
                process::exit(0);
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            "-d" | "--debug" => {
                options.verbose = true;
            }
            "-c" => {
                options.check_only = true;
            }
            "-W" => {
                options.warnings = true;
            }
            "-n" | "--dry-run" => {
                options.dry_run = true;
            }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("Error: -o requires an output file name".to_string());
                }
                options.output_path = Some(args[i].clone());
            }
            _ => {
                if arg.starts_with('-') {
                    return Err(format!("Error: Unknown option '{}'", arg));
                }
                if input_file.is_some() {
                    return Err("Error: Multiple input files specified".to_string());
                }
                input_file = Some(arg.clone());
            }
        }

        i += 1;
    }

    let input_file = input_file.ok_or("Error: No input file specified")?;

    // Warn if file doesn't have .peach extension
    if !input_file.ends_with(".peach") {
        eprintln!("Warning: Input file does not have .peach extension");
    }

    Ok((input_file, options))
}

fn print_usage() {
    println!("Usage: pc [options] <file.peach>");
    println!();
    println!("The Peach compiler - compiles Peach source files to C");
    println!();
    println!("Options:");
    println!("  -o <file>        Specify output file (default: <input>.c)");
    println!("  -c               Check syntax and types only, don't generate code");
    println!("  -d, --debug      Show compilation progress");
    println!("  -n, --dry-run    Print generated C code to stdout (don't write file)");
    println!("  -W               Enable warnings (placeholder for future)");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show compiler version");
}
