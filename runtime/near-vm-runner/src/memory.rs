use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_vm_logic::{MemSlice, MemoryLike};

use std::borrow::Cow;

use wasmer_runtime::units::Pages;
use wasmer_runtime::wasm::MemoryDescriptor;
use wasmer_runtime::Memory;

pub struct WasmerMemory(Memory);

impl WasmerMemory {
    pub fn new(initial_memory_pages: u32, max_memory_pages: u32) -> Self {
print_file_path_and_function_name!();

        WasmerMemory(
            Memory::new(
                MemoryDescriptor::new(
                    Pages(initial_memory_pages),
                    Some(Pages(max_memory_pages)),
                    false,
                )
                .unwrap(),
            )
            .expect("TODO creating memory cannot fail"),
        )
    }

    pub fn clone(&self) -> Memory {
print_file_path_and_function_name!();

        self.0.clone()
    }
}

impl WasmerMemory {
    fn with_memory<F, T>(&self, offset: u64, len: usize, func: F) -> Result<T, ()>
    where
        F: FnOnce(core::slice::Iter<'_, std::cell::Cell<u8>>) -> T,
    {
print_file_path_and_function_name!();

        let start = usize::try_from(offset).map_err(|_| ())?;
        let end = start.checked_add(len).ok_or(())?;
        self.0.view().get(start..end).map(|mem| func(mem.iter())).ok_or(())
    }
}

impl MemoryLike for WasmerMemory {
    fn fits_memory(&self, slice: MemSlice) -> Result<(), ()> {
print_file_path_and_function_name!();

        self.with_memory(slice.ptr, slice.len()?, |_| ())
    }

    fn view_memory(&self, slice: MemSlice) -> Result<Cow<[u8]>, ()> {
print_file_path_and_function_name!();

        self.with_memory(slice.ptr, slice.len()?, |mem| {
            Cow::Owned(mem.map(core::cell::Cell::get).collect())
        })
    }

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), ()> {
print_file_path_and_function_name!();

        self.with_memory(offset, buffer.len(), |mem| {
            buffer.iter_mut().zip(mem).for_each(|(dst, src)| *dst = src.get());
        })
    }

    fn write_memory(&mut self, offset: u64, buffer: &[u8]) -> Result<(), ()> {
print_file_path_and_function_name!();

        self.with_memory(offset, buffer.len(), |mem| {
            mem.zip(buffer.iter()).for_each(|(dst, src)| dst.set(*src));
        })
    }
}

#[test]
fn test_memory_like() {
    near_vm_logic::test_utils::test_memory_like(|| Box::new(WasmerMemory::new(1, 1)));
}
