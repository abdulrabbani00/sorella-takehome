# Database Optimization Challenge
The goal of this take home is to write a custom database implementation that 
properly stores data across restarts. The goal of this is to make the
implementation as fast as possible and obliterate the current implementation.

**Note:** You will have 2 days to complete the assignment from when you recieve the email containing it.

### Submission
1) Create a git repo with the solution and share it with our githubs: @jnoorchashm37 and @Will-Smith11
2) Send us an email with the subject line `Sorella Lab Rust Takehome - [Name]` at joseph@sorellalabs.xyz and will@sorellalabs.xyz

## Scope

### Task
1) Implement the most efficient `Database` that Impls `DbReader` and `DbWriter`
2) Implement or use default `TypeGenerator`
3) Make it as fast as possible!

### Code That Cannot Change
1) You cannot change the `RANGE_SIZE` constant and it must be used.
2) You cannot make any changes to the runner in `runner.rs` 
2) You cannot make any changes to the benchmark code in `benchmarks.rs` 
3) You cannot change any of the Trait definitions.
4) If using a Serializer/Deseralizer, you must use `serde_json`

### What Can Change
- You are allowed to change the `as_str` impl for the `DatabaseKey`
- You are allow to change `String` types to `&str` as-well as `Vec<>` to `&[]` for
defined types in `types.rs`
- Notably you cannot change it to be `&'static str` or `&'static []`, you must
use the lifetimes provided.

### Whats not allowed
1) static/global vars cannot be used.
2) multithreading
3) changing release profile
4) changing `global_allocator`
5) using `Box::leak` or anything else that leaks memory. 
6) no global memory
7) execution should theoretically work if the reads and writes happened on seperate processes (you cannot store pointers in the file and then read them to reconstruct active memory given the reads and writes happen in the same process). 


## NOTE 
Do not worry about the `DefaultDatabase` not working if you change any types,
this is expected behavior. Feel free to comment out. the `DefaultDatabase` is
a basic template for you to possibly use.


Please put your code under `src/user_solution/*`

