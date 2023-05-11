use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use wasmer::*;
use wasmer_engine::{Engine, Executable};
use wasmer_engine_universal::Universal;
use wasmer_vm::Artifact;

fn slow_to_compile_contract(n_fns: usize, n_locals: usize) -> Vec<u8> {
print_file_path_and_function_name!();

    let fns = format!("(func (local {}))\n", "i32 ".repeat(n_locals)).repeat(n_fns);
    let wat = format!(r#"(module {} (func (export "main")))"#, fns);
    wat2wasm(wat.as_bytes()).unwrap().to_vec()
}

fn compile_uncached<'a>(
    store: &'a Store,
    engine: &'a dyn Engine,
    code: &'a [u8],
    time: bool,
) -> Result<Box<dyn wasmer_engine::Executable>, CompileError> {
print_file_path_and_function_name!();

    use std::time::Instant;
    let now = Instant::now();
    engine.validate(code)?;
    let validate = now.elapsed().as_millis();
    let now = Instant::now();
    let res = engine.compile(code, store.tunables());
    let compile = now.elapsed().as_millis();
    if time {
        println!("validate {}ms compile {}ms", validate, compile);
    }
    res
}

#[test]
#[ignore]
fn compilation_test() {
print_file_path_and_function_name!();

    let compiler = Singlepass::default();
    let engine = Universal::new(compiler).engine();
    let store = Store::new(&engine);
    for factor in 1..1000 {
        let code = slow_to_compile_contract(3, 25 * factor);
        match compile_uncached(&store, &engine, &code, false) {
            Ok(art) => {
                let serialized = art.serialize().unwrap();
                println!(
                    "{}: artifact is compiled, size is {}",
                    factor,
                    serialized.len()
                );
            }
            Err(err) => {
                println!("err is {:?}", err);
            }
        }
    }
}

/*
Code to create perf map.

fn write_perf_profiler_map(functions: &Vec<NamedFunction>) -> Result<(), Box<dyn std::error::Error>>{
print_file_path_and_function_name!();

    let pid = process::id();
    let filename = format!("/tmp/perf-{}.map", pid);
    let mut file = File::create(filename).expect("Unable to create file");
    for f in functions {
        file.write_fmt(format_args!("{:x} {:x} {}\n", f.address, f.size, f.name))?;
    }
    Ok(())
}
*/

#[test]
fn profiling() {
    let wat = r#"
       (import "env" "impf" (func))
       (func $f0)
       (func (export "f1"))
       (func (export "f2"))
       (func (export "f3"))
    "#;
    let wasm = wat2wasm(wat.as_bytes()).unwrap();
    let compiler = Singlepass::default();
    let engine = Universal::new(compiler).engine();
    let store = Store::new(&engine);
    match compile_uncached(&store, &engine, &wasm, false) {
        Ok(art) => unsafe {
            let serialized = art.serialize().unwrap();
            let executable =
                wasmer_engine_universal::UniversalExecutableRef::deserialize(&serialized).unwrap();
            let artifact = engine.load_universal_executable_ref(&executable).unwrap();
            let info = artifact
                .functions()
                .iter()
                .filter_map(|(idx, _)| {
                    let extent = artifact.function_extent(idx)?;
                    let idx = artifact.import_counts().function_index(idx);
                    let name = executable.function_name(idx)?;
                    Some((name, extent))
                })
                .collect::<Vec<_>>();
            assert_eq!(4, info.len());
            assert_eq!("f0", info[0].0);
            assert_eq!("f1", info[1].0);
            assert_eq!("f2", info[2].0);
            assert_eq!("f3", info[3].0);
        },
        Err(_) => {
            assert!(false)
        }
    }
}