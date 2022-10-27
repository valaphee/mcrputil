// Copyright (c) 2022, Valaphee.
// All rights reserved.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {

}
