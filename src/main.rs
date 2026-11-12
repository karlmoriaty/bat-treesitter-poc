use lumis::highlight::{Budget, HighlightOptions};
use lumis::{languages::Language, themes, TerminalBuilder};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).unwrap_or_else(|| {
        eprintln!("usage: bat-ts-poc <file> [stream]");
        std::process::exit(2);
    });
    let stream = args.get(2).map(|s| s == "stream").unwrap_or(false);

    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read error: {e}");
        std::process::exit(1);
    });

    let t0 = Instant::now();
    let theme = themes::get("catppuccin_mocha").expect("theme");
    let formatter = TerminalBuilder::new()
        .language(Language::Rust)
        .theme(Some(theme))
        .build()
        .expect("formatter");
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let opts = HighlightOptions::new().budget(Budget::new().time_limit(None));

    let t1 = Instant::now();
    if stream {
        let file = std::fs::File::create("stream-out.txt").expect("create output");
        let mut writer = std::io::BufWriter::new(file);
        lumis::write_highlight_with_options(&mut writer, &src, formatter, opts)
            .expect("highlight");
        use std::io::Write;
        writer.flush().expect("flush");
        let ms = t1.elapsed().as_secs_f64() * 1000.0;
        println!("mode=stream formatter_build_ms={build_ms:.3} highlight_ms={ms:.3}");
        println!("input_bytes={}", src.len());
    } else {
        let out = lumis::highlight_with_options(&src, formatter, opts);
        let ms = t1.elapsed().as_secs_f64() * 1000.0;
        let esc = out.bytes().filter(|b| *b == 0x1b).count();
        println!("mode=string formatter_build_ms={build_ms:.3} highlight_ms={ms:.3}");
        println!("input_bytes={} output_bytes={} esc_count={esc}", src.len(), out.len());
    }
}