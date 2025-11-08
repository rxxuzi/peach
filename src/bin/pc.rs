use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // pc コマンドは引数なしでヘルプを表示
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let first_arg = &args[1];

    // オプションの処理
    match first_arg.as_str() {
        "--version" | "-v" => {
            println!("pc (Peach compiler) {}", VERSION);
            process::exit(0);
        }
        "--help" | "-h" => {
            print_usage();
            process::exit(0);
        }
        _ => {
            // それ以外はファイルとして扱う
            let input_file = first_arg;

            // .peach拡張子チェック（警告のみ）
            if !input_file.ends_with(".peach") {
                eprintln!("Warning: Input file does not have .peach extension");
            }

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
    }
}

fn print_usage() {
    println!("Usage: pc [options] <file.peach>");
    println!();
    println!("The Peach compiler - compiles Peach source files to C");
    println!();
    println!("Options:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show compiler version");
    println!();
    println!("Example:");
    println!("  pc hello.peach   Compile hello.peach to C");
}
