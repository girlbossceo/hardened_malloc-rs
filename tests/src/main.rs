use hardened_malloc_sys::HardenedMalloc;

#[global_allocator]
static GLOBAL: HardenedMalloc = HardenedMalloc; 

fn main() {
    let mut v = Vec::new();
    v.push(1);
    v.resize(5, 0);
    println!("v has value: {:?}", v);
}
