#![deny(unused_must_use)]
//use assembly::Assembly;
use rustc_codegen_clr::*;
use std::env;
fn load_ar(r: &mut impl std::io::Read) -> std::io::Result<assembly::Assembly> {
    use ar::Archive;
    use std::io::Read;
    let mut final_assembly = assembly::Assembly::empty();
    let mut archive = Archive::new(r);
    // Iterate over all entries in the archive:
    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result.unwrap();
        let name = String::from_utf8_lossy(entry.header().identifier());
        if name.contains(".bc") {
            let mut asm_bytes = Vec::with_capacity(0x100);
            entry
                .read_to_end(&mut asm_bytes)
                .expect("ERROR: Could not load the assembly file!");
            let assembly = postcard::from_bytes(&asm_bytes)
                .expect("ERROR:Could not decode the assembly file!");
            final_assembly = final_assembly.join(assembly);
        }
    }
    Ok(final_assembly)
}
enum AOTCompileMode {
    NoAOT,
    MonoAOT,
    FullMonoAOT,
}
impl AOTCompileMode {
    pub fn compile(self, path: &str) {
        match self {
            Self::NoAOT => (),
            Self::MonoAOT => {
                let out = std::process::Command::new("mono")
                    .arg("--aot")
                    .arg("-O=all")
                    .arg(path)
                    .output()
                    .expect("failed run mono AOT process");
            }
            Self::FullMonoAOT => {
                let out = std::process::Command::new("mono")
                    .arg("--aot=full")
                    .arg("-O=all")
                    .arg(path)
                    .output()
                    .expect("failed run mono AOT process");
            }
        }
    }
}
fn aot_compile_mode(args: &[String]) -> AOTCompileMode {
    if let Some(aot_idx) = args.iter().position(|arg| arg == "--aot_mode") {
        let aot_idx = aot_idx + 1;
        let aot = args
            .get(aot_idx)
            .expect("ERROR: \"--aot_mode\" provided, but no AOT mode set!");
        match aot.as_str() {
            "no" | "none" | "no_aot" | "no-aot" => AOTCompileMode::NoAOT,
            "mono" | "mono_aot" | "mono-aot" => AOTCompileMode::MonoAOT,
            "mono_full" | "mono-full" | "mono_full_aot" | "mono-full-aot" => {
                AOTCompileMode::FullMonoAOT
            }
            _ => panic!("Unknown AOT mode:{aot:?}"),
        }
    } else {
        AOTCompileMode::NoAOT
    }
}
fn main() {
    use std::io::Read;
    let args: Vec<String> = env::args().collect();
    let args = &args[1..];
    let to_link: Vec<_> = args.iter().filter(|arg| arg.contains(".bc")).collect();
    let ar_to_link: Vec<_> = args.iter().filter(|arg| arg.contains(".rlib")).collect();
    let output = &args[1 + args
        .iter()
        .position(|arg| arg == "-o")
        .expect("No output file!")];
    let mut final_assembly = assembly::Assembly::empty();
    for asm_path in &to_link {
        let mut asm_file =
            std::fs::File::open(asm_path).expect("ERROR:Could not open the assembly file!");
        let mut asm_bytes = Vec::with_capacity(0x100);
        asm_file
            .read_to_end(&mut asm_bytes)
            .expect("ERROR: Could not load the assembly file!");
        let assembly =
            postcard::from_bytes(&asm_bytes).expect("ERROR:Could not decode the assembly file!");
        final_assembly = final_assembly.join(assembly);
    }
    for asm_path in &ar_to_link {
        let mut asm_file =
            std::fs::File::open(asm_path).expect("ERROR: Could not open the assembly file!");
        let assembly = load_ar(&mut asm_file).expect("Could not oper archive");
        final_assembly = final_assembly.join(assembly);
    }
    libc::insert_libc(&mut final_assembly);
    use rustc_codegen_clr::assembly_exporter::AssemblyExporter;
    let path = output;
    let is_lib = output.contains(".dll") || output.contains(".so") || output.contains(".o");
    rustc_codegen_clr::assembly_exporter::ilasm_exporter::ILASMExporter::export_assembly(
        &final_assembly,
        path.as_ref(),
        is_lib,
    )
    .expect("Assembly export faliure!");
}
