pub use backtrace::Backtrace;
pub use chrono::Utc;
pub use lazy_static::lazy_static;
pub use std::collections::HashMap;
pub use std::sync::Mutex;

lazy_static! {
    pub static ref TOTAL_COUNTER: Mutex<i32> = Mutex::new(0);
    pub static ref DEPTH_COUNTER: Mutex<usize> = Mutex::new(0);
    pub static ref FN_COUNT_MAP: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    pub static ref PATH_COUNT_MAP: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub struct DepthGuard;

impl Drop for DepthGuard {
    fn drop(&mut self) {
        let mut depth = DEPTH_COUNTER.lock().unwrap();
        *depth -= 1;
    }
}

#[macro_export]
macro_rules! print_file_path_and_function_name {
    () => {
        {
        let mut depth = DEPTH_COUNTER.lock().unwrap();
            *depth += 1;
            let current_depth = *depth;

            {
                fn f() {}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                let function_name = match &name[..name.len() - 3].rfind(':') {
                    Some(pos) => &name[pos + 1..name.len() - 3],
                    None => &name[..name.len() - 3],
                };

                let impl_name = match &name[..name.len() - 3].rfind('<') {
                    Some(pos) => {
                        let end_pos = &name[..name.len() - 3].rfind('>').unwrap();
                        &name[pos + 1..*end_pos]
                    }
                    None => "",
                };

                let impl_parts: Vec<&str> = impl_name.split(" as ").collect();
                let impl_type = impl_parts.get(0).unwrap_or(&"").rsplit("::").next().unwrap_or("");
                let impl_trait = impl_parts.get(1).unwrap_or(&"").rsplit("::").next().unwrap_or("");

                let depth_output = if current_depth > 1 {
                    format!("depth: {}", current_depth - 1)
                } else {
                    "".to_string()
                };

                let real_file_path = format!("{}", file!());
                let file_path = format!("{}(), {}", function_name, file!());

                let mut path_count_map = PATH_COUNT_MAP.lock().unwrap();
                let mut fn_count_map = FN_COUNT_MAP.lock().unwrap();

                // 그 경로가 출력된 횟수
                let path_count = path_count_map.entry(real_file_path.clone()).or_insert(0);
                *path_count += 1;

                let t = format!("{} of {} as {}",function_name,impl_type, impl_trait);
                // 그 함수가 출력된 횟수
                let fn_count = fn_count_map.entry(t.clone()).or_insert(0);
                *fn_count += 1;


                let mut thread_id_string = format!("{:?}", std::thread::current().id());
                let thread_id_usize = thread_id_string.trim_start_matches("ThreadId(").trim_end_matches(')').parse::<usize>().unwrap();

                println!(
                    "cnt({:?}) path-cnt({}) fn-cnt({}) [{}] {} {}: {}() {}",
                    TOTAL_COUNTER.lock().unwrap(),
                    path_count,// 그 경로가 출력된 횟수
                    fn_count,// 그 함수가 출력된 횟수
                    thread_id_usize, // ThreadId(1)을 (1)로 변환
                    file!(),
                    line!(),
                    function_name,
                    if impl_name.is_empty() { "".to_string() }
                    else { format!("impl: {} as {}", impl_type, impl_trait) }, // impl 출력
                );
                let mut i = TOTAL_COUNTER.lock().unwrap();
                *i += 1;
            }
            DepthGuard
        }
    };
}


// pub use backtrace::Backtrace;
// pub use chrono::Utc;
// pub use lazy_static::lazy_static;
// pub use std::sync::Mutex;
//
// lazy_static! {
//     pub static ref TOTAL_COUNTER: Mutex<i32> = Mutex::new(0);
//     pub static ref DEPTH_COUNTER: Mutex<usize> = Mutex::new(0);
// }
//
// pub struct DepthGuard;
//
// impl Drop for DepthGuard {
//     fn drop(&mut self) {
//         let mut depth = DEPTH_COUNTER.lock().unwrap();
//         *depth -= 1;
//     }
// }
//
// #[macro_export]
// macro_rules! print_file_path_and_function_name {
//     () => {
//         {
//             let mut depth = DEPTH_COUNTER.lock().unwrap();
//             *depth += 1;
//             let current_depth = *depth;
//
//             {
//                 fn f() {}
//                 fn type_name_of<T>(_: T) -> &'static str {
//                     std::any::type_name::<T>()
//                 }
//                 let name = type_name_of(f);
//                 let function_name = match &name[..name.len() - 3].rfind(':') {
//                     Some(pos) => &name[pos + 1..name.len() - 3],
//                     None => &name[..name.len() - 3],
//                 };
//
//                 let depth_output = if current_depth > 1 {
//                     format!("depth: {}", current_depth - 1)
//                 } else {
//                     "".to_string()
//                 };
//
//                 println!(
//                     "{:?}, {:?}, {}, {}-{}, {}",
//                     TOTAL_COUNTER.lock().unwrap(),
//                     std::thread::current().id(),
//                     format!("{}/\n     fn : {}", file!(), function_name),
//                     line!(),
//                     column!(),
//                     depth_output
//                 );
//      // println!(
//      //                "{:?}, {:?}, {:?}, {}, {}-{}, {}",
//      //                TOTAL_COUNTER.lock().unwrap(),
//      //                Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
//      //                std::thread::current().id(),
//      //                format!("{}/ fn : {}", file!(), function_name),
//      //                line!(),
//      //                column!(),
//      //                depth_output
//      //            );
//
//                 let mut i = TOTAL_COUNTER.lock().unwrap();
//                 *i += 1;
//             }
//             DepthGuard
//         }
//     };
// }